use crate::feedback::{LetterStatus, evaluate_feedback, statuses_from_pattern};
use crate::filter::{GuessInput, filter_candidates, is_candidate_consistent};
use crate::lexicon::Lexicon;
use crate::past_solutions::{PastSolutionEntry, PastSolutionIndex};
use crate::rank::{InformationGuess, LikelyAnswer, rank_information_guesses, rank_likely_answers};
use crate::word::Word;
use std::io;
use std::path::Path;

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

#[derive(Clone, Debug)]
pub struct Solver {
    lexicon: Lexicon,
    past: PastSolutionIndex,
}

impl Solver {
    pub fn load(data_dir: impl AsRef<Path>) -> io::Result<Self> {
        let data_dir = data_dir.as_ref();
        Ok(Self {
            lexicon: Lexicon::load(data_dir)?,
            past: PastSolutionIndex::load(data_dir.join("past_solutions.json"))?,
        })
    }

    pub fn new(lexicon: Lexicon, past: PastSolutionIndex) -> Self {
        Self { lexicon, past }
    }

    pub fn solve(&self, request: &SolveRequest) -> Result<SolveResponse, String> {
        let candidates = filter_candidates(&self.lexicon.candidate_solutions, &request.guesses);
        if candidates.is_empty() {
            return Err("No candidate solution matches the provided guesses and statuses. Check duplicate-letter feedback.".to_string());
        }

        let likely_answers = rank_likely_answers(
            &candidates,
            &self.past,
            &request.past_solution_policy,
            &self.lexicon.overrides,
        );
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
        let mut best_information_guesses =
            rank_information_guesses(&legal_guesses, &candidates, &likely_answers, &self.past);

        match request.mode {
            SolveMode::LikelyAnswer => best_information_guesses.clear(),
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

    pub fn health_json(&self) -> String {
        let (past_solution_first_date, past_solution_latest_date) =
            self.past.date_range().unwrap_or(("", ""));
        format!(
            "{{\"ok\":true,\"candidate_solutions\":{},\"allowed_guesses\":{},\"past_solutions\":{},\"past_solution_first_date\":\"{}\",\"past_solution_latest_date\":\"{}\",\"data_updated_at\":\"fixture\"}}",
            self.lexicon.candidate_solutions.len(),
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
                            .find(|guess| guess.is_possible_answer)
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
    .map(|(date, puzzle_number, answer)| BacktestCase {
        date: date.to_string(),
        puzzle_number,
        answer: Word::parse(answer).expect("valid backtest answer"),
    })
    .collect()
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
