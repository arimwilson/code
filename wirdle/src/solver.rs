use crate::feedback::{LetterStatus, evaluate_feedback, statuses_from_pattern};
use crate::filter::{GuessInput, filter_candidates, is_candidate_consistent};
use crate::lexicon::Lexicon;
use crate::past_solutions::{PastSolutionEntry, PastSolutionIndex};
use crate::rank::{
    CandidatePool, GuessStats, InformationGuess, LikelyAnswer, bucket_stats, information_guess,
    rank_information_guesses, rank_likely_answers, sort_information_guesses,
};
use crate::word::Word;
use std::io;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolveMode {
    LikelyAnswer,
    MaxInformation,
    Minimax,
    Hybrid,
}

impl SolveMode {
    pub fn parse(input: &str) -> Self {
        match input {
            "likely_answer" => Self::LikelyAnswer,
            "max_information" => Self::MaxInformation,
            "minimax" => Self::Minimax,
            _ => Self::Hybrid,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PastSolutionPolicy {
    pub enabled: bool,
    pub weight_multiplier: f64,
    pub recent_repeat_multiplier: f64,
    pub recent_days: usize,
}

impl Default for PastSolutionPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            weight_multiplier: 0.05,
            recent_repeat_multiplier: 0.01,
            recent_days: 90,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SolveRequest {
    pub guesses: Vec<GuessInput>,
    pub mode: SolveMode,
    pub hard_mode: bool,
    pub limit: usize,
    pub past_solution_policy: PastSolutionPolicy,
}

#[derive(Clone, Debug)]
pub struct SolveResponse {
    pub remaining_candidates: usize,
    pub likely_answers: Vec<LikelyAnswer>,
    pub best_information_guesses: Vec<InformationGuess>,
}

/// First-turn statistics for every allowed guess, parallel to
/// `Lexicon::allowed_guesses`.
///
/// With the accepted-guess list as the solution universe, the empty-board
/// entropy sweep is ~14,855 x 14,855 feedback evaluations (~15s). Those numbers
/// depend only on the word lists, not on the request, so they are computed once
/// at startup and reused by every fresh game.
#[derive(Clone, Debug)]
pub struct FirstTurnStats {
    stats: Vec<GuessStats>,
}

impl FirstTurnStats {
    pub fn compute(lexicon: &Lexicon) -> Self {
        // The empty-board pool is the whole universe. Answer probabilities are
        // request-dependent, so they are supplied later; only the bucket
        // statistics are cached here.
        let pool = CandidatePool::new(&lexicon.allowed_guesses, lexicon, &[]);
        let stats = lexicon
            .allowed_guesses
            .iter()
            .map(|guess| bucket_stats(*guess, &pool))
            .collect();
        Self { stats }
    }

    fn stats(&self) -> &[GuessStats] {
        &self.stats
    }
}

#[derive(Clone, Debug)]
pub struct Solver {
    lexicon: Lexicon,
    past: PastSolutionIndex,
    first_turn: Option<Arc<FirstTurnStats>>,
}

impl Solver {
    /// Load a solver. Call `with_first_turn_cache` to precompute the empty-board
    /// statistics; the server does so before binding its listener, so no request
    /// can arrive before the cache is ready.
    pub fn load(data_dir: impl AsRef<Path>) -> io::Result<Self> {
        let data_dir = data_dir.as_ref();
        Ok(Self {
            lexicon: Lexicon::load(data_dir)?,
            past: PastSolutionIndex::load(data_dir.join("past_solutions.json"))?,
            first_turn: None,
        })
    }

    pub fn new(lexicon: Lexicon, past: PastSolutionIndex) -> Self {
        Self {
            lexicon,
            past,
            first_turn: None,
        }
    }

    /// Precompute the empty-board statistics. Blocking and slow (seconds over
    /// the full accepted-guess list) — callers run it before serving traffic.
    pub fn with_first_turn_cache(mut self) -> Self {
        let start = Instant::now();
        self.first_turn = Some(Arc::new(FirstTurnStats::compute(&self.lexicon)));
        eprintln!(
            "precomputed first-turn statistics for {} guesses in {:?}",
            self.lexicon.allowed_guesses.len(),
            start.elapsed()
        );
        self
    }

    pub fn solve(&self, request: &SolveRequest) -> Result<SolveResponse, String> {
        let candidates = filter_candidates(&self.lexicon.allowed_guesses, &request.guesses);
        if candidates.is_empty() {
            return Err("No candidate solution matches the provided guesses and statuses. Check duplicate-letter feedback.".to_string());
        }

        let likely_answers = rank_likely_answers(
            &candidates,
            &self.past,
            &request.past_solution_policy,
            &self.lexicon,
        );
        // LikelyAnswer mode discards the information ranking, so skip building it.
        let mut best_information_guesses = if request.mode == SolveMode::LikelyAnswer {
            Vec::new()
        } else if let Some(cached) = self.cached_first_turn(request, &likely_answers) {
            cached
        } else {
            let legal_guesses: Vec<Word> = if request.hard_mode {
                self.lexicon
                    .allowed_guesses
                    .iter()
                    .copied()
                    .filter(|word| is_candidate_consistent(*word, &request.guesses))
                    .collect()
            } else {
                self.lexicon.allowed_guesses.clone()
            };
            rank_information_guesses(
                &legal_guesses,
                &candidates,
                &likely_answers,
                &self.past,
                &self.lexicon,
            )
        };

        match request.mode {
            SolveMode::LikelyAnswer => {}
            SolveMode::MaxInformation | SolveMode::Hybrid => {}
            SolveMode::Minimax => best_information_guesses.sort_by(|a, b| {
                a.worst_case_remaining
                    .cmp(&b.worst_case_remaining)
                    .then_with(|| b.entropy_bits.total_cmp(&a.entropy_bits))
                    .then_with(|| a.word.cmp(&b.word))
            }),
        }

        let limit = request.limit.max(1);
        Ok(SolveResponse {
            remaining_candidates: candidates.len(),
            likely_answers: likely_answers.into_iter().take(limit).collect(),
            best_information_guesses: best_information_guesses.into_iter().take(limit).collect(),
        })
    }

    /// Build the first-turn information ranking from cached bucket statistics.
    ///
    /// Only the score mixes in request-dependent terms, so the expensive bucket
    /// sweep is reused across every fresh game regardless of mode or
    /// past-solution policy. Returns `None` unless the board is empty and the
    /// cache is present.
    fn cached_first_turn(
        &self,
        request: &SolveRequest,
        likely_answers: &[LikelyAnswer],
    ) -> Option<Vec<InformationGuess>> {
        if !request.guesses.is_empty() {
            return None;
        }
        let cache = self.first_turn.as_ref()?;

        let pool = CandidatePool::new(&self.lexicon.allowed_guesses, &self.lexicon, likely_answers);
        // Zipping keeps the cache aligned with the word list structurally, so a
        // mismatch cannot silently drop guesses from the ranking.
        let mut ranked: Vec<InformationGuess> = self
            .lexicon
            .allowed_guesses
            .iter()
            .copied()
            .zip(cache.stats().iter())
            .map(|(guess, stats)| information_guess(guess, stats, &pool, &self.past, &self.lexicon))
            .collect();
        sort_information_guesses(&mut ranked);
        Some(ranked)
    }

    pub fn health_json(&self) -> String {
        let (past_solution_first_date, past_solution_latest_date) =
            self.past.date_range().unwrap_or(("", ""));
        format!(
            "{{\"ok\":true,\"likelier_solutions\":{},\"allowed_guesses\":{},\"past_solutions\":{},\"past_solution_first_date\":\"{}\",\"past_solution_latest_date\":\"{}\",\"data_updated_at\":\"fixture\"}}",
            self.lexicon.likelier_solutions.len(),
            self.lexicon.allowed_guesses.len(),
            self.past.entries().len(),
            past_solution_first_date,
            past_solution_latest_date
        )
    }

    pub fn metadata_json(&self) -> String {
        let policy = PastSolutionPolicy::default();
        format!(
            "{{\"data_version\":\"fixture\",\"supports_repeated_answers\":true,\"default_past_solution_policy\":{{\"enabled\":{},\"weight_multiplier\":{},\"recent_repeat_multiplier\":{},\"recent_days\":{}}}}}",
            policy.enabled,
            policy.weight_multiplier,
            policy.recent_repeat_multiplier,
            policy.recent_days
        )
    }

    pub fn lexicon(&self) -> &Lexicon {
        &self.lexicon
    }

    pub fn past(&self) -> &PastSolutionIndex {
        &self.past
    }
}

#[derive(Clone, Debug)]
pub struct BacktestCase {
    pub date: String,
    pub puzzle_number: u32,
    pub answer: Word,
}

#[derive(Clone, Debug)]
pub struct BacktestGame {
    pub case: BacktestCase,
    pub guesses: Vec<Word>,
    pub solved: bool,
}

pub fn run_backtest(
    lexicon: &Lexicon,
    past: &PastSolutionIndex,
    cases: &[BacktestCase],
    max_turns: usize,
) -> Vec<BacktestGame> {
    cases
        .iter()
        .map(|case| {
            let as_of_past = past.as_of_before(&case.date);
            let solver = Solver::new(lexicon.clone(), as_of_past);
            let mut guesses = Vec::new();
            let mut observed = Vec::new();

            for turn in 0..max_turns {
                let guess = if turn == 0 {
                    Word::parse("slate").expect("valid opener")
                } else {
                    let request = SolveRequest {
                        guesses: observed.clone(),
                        mode: SolveMode::Hybrid,
                        hard_mode: false,
                        limit: 20,
                        past_solution_policy: PastSolutionPolicy {
                            enabled: false,
                            ..PastSolutionPolicy::default()
                        },
                    };
                    let response = solver.solve(&request).expect("backtest candidates remain");
                    if response.remaining_candidates <= 2 {
                        response.likely_answers[0].word
                    } else {
                        response
                            .best_information_guesses
                            .iter()
                            .find(|guess| guess.is_possible_answer && guess.is_likelier)
                            .or_else(|| {
                                response
                                    .best_information_guesses
                                    .iter()
                                    .find(|guess| guess.is_possible_answer)
                            })
                            .or_else(|| response.best_information_guesses.first())
                            .map(|guess| guess.word)
                            .expect("information guess")
                    }
                };

                guesses.push(guess);
                if guess == case.answer {
                    return BacktestGame {
                        case: case.clone(),
                        guesses,
                        solved: true,
                    };
                }

                let pattern = evaluate_feedback(guess, case.answer);
                observed.push(GuessInput::new(guess, statuses_from_pattern(pattern)));
            }

            BacktestGame {
                case: case.clone(),
                guesses,
                solved: false,
            }
        })
        .collect()
}

pub fn known_backtest_cases() -> Vec<BacktestCase> {
    [
        ("2026-05-25", 1801, "visit"),
        ("2026-05-26", 1802, "couch"),
        ("2026-05-27", 1803, "stuff"),
        ("2026-05-28", 1804, "divot"),
        ("2026-05-29", 1805, "clang"),
    ]
    .into_iter()
    .chain(off_list_backtest_cases())
    .map(|(date, puzzle_number, answer)| BacktestCase {
        date: date.to_string(),
        puzzle_number,
        answer: Word::parse(answer).expect("valid backtest answer"),
    })
    .collect()
}

/// Answers NYT chose that were not on the likelier-solutions list when they ran.
///
/// These were unsolvable while that list was the candidate universe, so they are
/// the regression cases for the accepted-list universe.
pub fn off_list_backtest_cases() -> Vec<(&'static str, u32, &'static str)> {
    vec![
        ("2026-07-15", 1852, "pshaw"),
        ("2026-07-21", 1858, "shill"),
        ("2026-07-25", 1862, "aloha"),
        ("2026-08-09", 1877, "clunk"),
        ("2026-08-14", 1882, "geode"),
        ("2026-08-16", 1884, "aspic"),
        ("2026-08-24", 1892, "runny"),
        ("2026-08-26", 1894, "capon"),
    ]
}

pub fn entry(date: &str, puzzle_number: u32, solution: &str) -> PastSolutionEntry {
    PastSolutionEntry {
        date: date.to_string(),
        puzzle_number,
        solution: Word::parse(solution).expect("valid solution"),
        source: "fixture".to_string(),
        is_repeat: false,
    }
}

pub fn statuses_json(statuses: &[LetterStatus; 5]) -> String {
    let values: Vec<String> = statuses
        .iter()
        .map(|status| format!("\"{}\"", status.as_str()))
        .collect();
    format!("[{}]", values.join(","))
}
