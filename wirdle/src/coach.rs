use crate::feedback::LetterStatus;
use crate::filter::{GuessInput, filter_candidates, is_candidate_consistent};
use crate::rank::{
    CandidatePool, bucket_stats, effective_size, evaluate_information_guess, information_guess,
};
use crate::rank::{InformationGuess, rank_likely_answers};
use crate::solver::{PastSolutionPolicy, Solver};
use crate::word::Word;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoachIntent {
    PostGameReview,
    EasyHint,
}

impl CoachIntent {
    pub fn parse(input: &str) -> Option<Self> {
        match input {
            "post_game_review" => Some(Self::PostGameReview),
            "easy_hint" => Some(Self::EasyHint),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::PostGameReview => "post_game_review",
            Self::EasyHint => "easy_hint",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CoachRequest {
    pub intent: CoachIntent,
    pub guesses: Vec<GuessInput>,
    pub hard_mode: bool,
    pub hint_request: Option<HintRequest>,
    pub session_context: Option<SessionContext>,
}

#[derive(Clone, Debug, Default)]
pub struct HintRequest {
    pub requested_level: Option<u8>,
    pub explain_current: bool,
    pub confirmed_spoiler: bool,
}

#[derive(Clone, Debug, Default)]
pub struct SessionContext {
    pub highest_hint_level_used: u8,
    pub hint_levels_used: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoardState {
    InProgress,
    Solved { turn: usize },
    Lost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InformationBucket {
    Low,
    Modest,
    Solid,
    Sharp,
}

impl InformationBucket {
    fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Modest => "modest",
            Self::Solid => "solid",
            Self::Sharp => "sharp",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstraintDiscipline {
    Clean,
    Miss,
}

impl ConstraintDiscipline {
    fn as_str(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Miss => "miss",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveType {
    Probe,
    SolveAttempt,
    ForcedSolve,
    ConstraintMiss,
}

impl MoveType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Probe => "probe",
            Self::SolveAttempt => "solve_attempt",
            Self::ForcedSolve => "forced_solve",
            Self::ConstraintMiss => "constraint_miss",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Opener,
    Middle,
    Endgame,
    Final,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TrapRisk {
    Low,
    Moderate,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DuplicateLetterNote {
    None,
    Useful,
    Risky,
}

#[derive(Clone, Debug)]
pub struct BoardAnalysis {
    pub state: BoardState,
    pub turns: Vec<TurnAnalysis>,
    pub final_remaining_candidates: usize,
    /// Prior-weighted size of the final pool. Thresholds tuned against the old
    /// answer-only universe compare against this, not the raw count: the
    /// accepted-guess universe adds ~12.5k words worth a fraction of an answer
    /// each, which would otherwise push every board into a lower risk tier.
    pub final_effective_candidates: f64,
}

#[derive(Clone, Debug)]
pub struct TurnAnalysis {
    pub turn_index: usize,
    pub statuses: [LetterStatus; 5],
    pub candidates_before: usize,
    pub candidates_after: usize,
    pub information: InformationGuess,
    pub information_bucket: InformationBucket,
    pub constraint_discipline: ConstraintDiscipline,
    pub constraint_note: Option<String>,
    pub move_type: MoveType,
    stage: Stage,
    trap_risk: TrapRisk,
    duplicate_letter_note: DuplicateLetterNote,
    vowel_count: usize,
    solved_on_turn: bool,
}

#[derive(Clone, Debug)]
pub struct CoachResponse {
    pub intent: CoachIntent,
    pub board: BoardSummary,
    pub post_game: Option<PostGameReport>,
    pub easy_hint: Option<HintResponse>,
    pub share: Option<ShareOutput>,
}

#[derive(Clone, Debug)]
pub struct BoardSummary {
    pub state: BoardState,
    pub turns: usize,
    pub remaining_candidates: usize,
}

#[derive(Clone, Debug)]
pub struct PostGameReport {
    pub grades: Vec<String>,
    pub turns: Vec<TurnReview>,
    pub summary: GameSummary,
}

#[derive(Clone, Debug)]
pub struct TurnReview {
    pub turn: usize,
    pub label: String,
    pub grade: String,
    pub score: i32,
    pub move_type: MoveType,
    pub information: InformationBucket,
    pub constraint_discipline: ConstraintDiscipline,
    pub did_well: String,
    pub missed: String,
    pub summary: String,
}

#[derive(Clone, Debug)]
pub struct GameSummary {
    pub best_move_turn: usize,
    pub most_questionable_turn: usize,
    pub biggest_information_gain_turn: usize,
    pub best_recovery_turn: Option<usize>,
    pub missed_opportunity: Option<String>,
    pub lesson: String,
}

#[derive(Clone, Debug)]
pub struct ShareOutput {
    pub text: String,
    pub contains_guess_words: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HintLevel {
    GentleNudge = 1,
    NextMoveStrategy = 2,
    UsefulLetter = 3,
    Pattern = 4,
    StrongGuessHelp = 5,
    AnswerReveal = 6,
}

impl HintLevel {
    fn from_requested(value: u8) -> Self {
        match value.clamp(1, 6) {
            1 => Self::GentleNudge,
            2 => Self::NextMoveStrategy,
            3 => Self::UsefulLetter,
            4 => Self::Pattern,
            5 => Self::StrongGuessHelp,
            _ => Self::AnswerReveal,
        }
    }

    fn as_u8(self) -> u8 {
        self as u8
    }

    fn label(self) -> &'static str {
        match self {
            Self::GentleNudge => "Gentle Nudge",
            Self::NextMoveStrategy => "Next-Move Strategy",
            Self::UsefulLetter => "Useful Letter",
            Self::Pattern => "Pattern",
            Self::StrongGuessHelp => "Strong Guess Help",
            Self::AnswerReveal => "Answer Reveal",
        }
    }

    fn next_action_label(self) -> Option<&'static str> {
        match self {
            Self::GentleNudge => Some("A little more"),
            Self::NextMoveStrategy => Some("Show useful letters"),
            Self::UsefulLetter => Some("Show a pattern"),
            Self::Pattern => Some("Help me choose"),
            Self::StrongGuessHelp => Some("Reveal answer"),
            Self::AnswerReveal => None,
        }
    }

    fn spoiler_risk(self) -> SpoilerRisk {
        match self {
            Self::GentleNudge => SpoilerRisk::Low,
            Self::NextMoveStrategy => SpoilerRisk::LowMedium,
            Self::UsefulLetter => SpoilerRisk::MediumHigh,
            Self::Pattern => SpoilerRisk::High,
            Self::StrongGuessHelp => SpoilerRisk::VeryHigh,
            Self::AnswerReveal => SpoilerRisk::Complete,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpoilerRisk {
    Low,
    LowMedium,
    MediumHigh,
    High,
    VeryHigh,
    Complete,
}

impl SpoilerRisk {
    fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::LowMedium => "low_medium",
            Self::MediumHigh => "medium_high",
            Self::High => "high",
            Self::VeryHigh => "very_high",
            Self::Complete => "complete",
        }
    }
}

#[derive(Clone, Debug)]
pub struct HintResponse {
    pub level: HintLevel,
    pub label: &'static str,
    pub spoiler_risk: SpoilerRisk,
    pub message: String,
    pub rationale: Option<String>,
    pub next_action_label: Option<&'static str>,
    pub requires_confirmation_for_next: bool,
    pub revealed_words: Vec<Word>,
    pub share_summary: HintShareSummary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HintShareSummary {
    pub highest_hint_level_used: u8,
    pub hint_labels_used: Vec<String>,
}

#[derive(Clone, Debug)]
struct KnownYellow {
    letter: u8,
    blocked_positions: Vec<usize>,
}

#[derive(Clone, Debug)]
struct HintGuessOption {
    word: Word,
    information: InformationGuess,
    human_score: i32,
    explanation: String,
}

#[derive(Clone, Debug)]
pub struct CoachError {
    pub code: &'static str,
    pub message: String,
}

impl CoachError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub fn coach(solver: &Solver, request: &CoachRequest) -> Result<CoachResponse, CoachError> {
    match request.intent {
        CoachIntent::PostGameReview => post_game_review(solver, request),
        CoachIntent::EasyHint => easy_hint(solver, request),
    }
}

pub fn analyze_board(
    solver: &Solver,
    guesses: &[GuessInput],
    hard_mode: bool,
) -> Result<BoardAnalysis, CoachError> {
    let mut turns = Vec::with_capacity(guesses.len());
    let mut prior = Vec::new();
    let mut solved_turn = None;
    let mut final_remaining_candidates = solver.lexicon().allowed_guesses.len();
    let mut final_effective_candidates =
        effective_size(&solver.lexicon().allowed_guesses, solver.lexicon());

    for (idx, guess) in guesses.iter().enumerate() {
        if solved_turn.is_some() {
            return Err(CoachError::new(
                "invalid_request",
                "Rows after a solved Wordle row are not valid game data.",
            ));
        }

        let candidates_before = filter_candidates(&solver.lexicon().allowed_guesses, &prior);
        if candidates_before.is_empty() {
            return Err(board_inconsistent());
        }

        let likely_answers = rank_likely_answers(
            &candidates_before,
            solver.past(),
            &PastSolutionPolicy::default(),
            solver.lexicon(),
        );
        let information = evaluate_information_guess(
            guess.word,
            &candidates_before,
            &likely_answers,
            solver.past(),
            solver.lexicon(),
        );
        let mut with_current = prior.clone();
        with_current.push(guess.clone());
        let candidates_after = filter_candidates(&solver.lexicon().allowed_guesses, &with_current);
        if candidates_after.is_empty() {
            return Err(board_inconsistent());
        }

        let respects_known_info = prior.is_empty() || is_candidate_consistent(guess.word, &prior);
        let constraint_note = constraint_note(guess, &prior, hard_mode, respects_known_info);
        let solved_on_turn = guess
            .statuses
            .iter()
            .all(|status| *status == LetterStatus::Correct);
        if solved_on_turn {
            solved_turn = Some(idx + 1);
        }

        final_remaining_candidates = candidates_after.len();
        final_effective_candidates = effective_size(&candidates_after, solver.lexicon());
        turns.push(TurnAnalysis {
            turn_index: idx,
            statuses: guess.statuses,
            candidates_before: candidates_before.len(),
            candidates_after: candidates_after.len(),
            information,
            information_bucket: information_bucket(candidates_before.len(), candidates_after.len()),
            constraint_discipline: if respects_known_info {
                ConstraintDiscipline::Clean
            } else {
                ConstraintDiscipline::Miss
            },
            constraint_note,
            move_type: move_type(
                respects_known_info,
                candidates_before.len(),
                idx,
                information_bucket(candidates_before.len(), candidates_after.len()),
                guess.word,
                &candidates_before,
                solver.lexicon().is_likelier(guess.word),
            ),
            stage: stage(idx),
            trap_risk: trap_risk(
                &candidates_before,
                effective_size(&candidates_before, solver.lexicon()),
            ),
            duplicate_letter_note: duplicate_letter_note(guess),
            vowel_count: vowel_count(guess.word),
            solved_on_turn,
        });
        prior = with_current;
    }

    let state = if let Some(turn) = solved_turn {
        BoardState::Solved { turn }
    } else if guesses.len() == 6 {
        BoardState::Lost
    } else {
        BoardState::InProgress
    };

    Ok(BoardAnalysis {
        state,
        turns,
        final_remaining_candidates,
        final_effective_candidates,
    })
}

fn post_game_review(solver: &Solver, request: &CoachRequest) -> Result<CoachResponse, CoachError> {
    let analysis = analyze_board(solver, &request.guesses, request.hard_mode)?;
    if analysis.state == BoardState::InProgress {
        return Err(CoachError::new(
            "board_incomplete",
            "Finish the Wordle board before reviewing your game.",
        ));
    }

    let report = build_post_game_report(&analysis);
    let hint_summary = hint_share_summary(request.session_context.as_ref(), None);
    let share_text = build_post_game_share_text(
        &analysis,
        &report,
        (hint_summary.highest_hint_level_used > 0).then_some(&hint_summary),
    );
    let contains_guess_words = request
        .guesses
        .iter()
        .any(|guess| share_text.to_lowercase().contains(guess.word.as_str()));

    Ok(CoachResponse {
        intent: request.intent,
        board: BoardSummary {
            state: analysis.state.clone(),
            turns: analysis.turns.len(),
            remaining_candidates: analysis.final_remaining_candidates,
        },
        post_game: Some(report),
        easy_hint: None,
        share: Some(ShareOutput {
            text: share_text,
            contains_guess_words,
        }),
    })
}

fn easy_hint(solver: &Solver, request: &CoachRequest) -> Result<CoachResponse, CoachError> {
    let analysis = analyze_board(solver, &request.guesses, request.hard_mode)?;
    if analysis.state != BoardState::InProgress {
        return Err(CoachError::new(
            "game_finished",
            "This Wordle is already complete. Switch to Post Game for a review.",
        ));
    }

    let candidates = current_candidates(solver, &request.guesses);
    if candidates.is_empty() {
        return Err(board_inconsistent());
    }

    let hint_level = choose_hint_level(
        &analysis,
        request.hint_request.as_ref(),
        request.session_context.as_ref(),
    );
    let hint_request = request.hint_request.as_ref().cloned().unwrap_or_default();
    if matches!(
        hint_level,
        HintLevel::StrongGuessHelp | HintLevel::AnswerReveal
    ) && !hint_request.confirmed_spoiler
    {
        return Err(CoachError::new(
            "spoiler_confirmation_required",
            "This hint can reveal answer-like words. Confirm that you want a stronger hint.",
        ));
    }
    if hint_level == HintLevel::AnswerReveal && candidates.len() != 1 {
        return Err(CoachError::new(
            "answer_reveal_unavailable",
            "Wirdle cannot identify one exact answer from this board yet. Ask for strong guess help instead.",
        ));
    }

    let hint = build_hint_response(
        solver,
        request,
        &analysis,
        &candidates,
        hint_level,
        hint_request.explain_current,
    );
    let share_text = build_easy_share_text(&analysis, &hint.share_summary);
    let contains_guess_words = request
        .guesses
        .iter()
        .map(|guess| guess.word)
        .chain(hint.revealed_words.iter().copied())
        .any(|word| share_text.to_lowercase().contains(word.as_str()));

    Ok(CoachResponse {
        intent: request.intent,
        board: BoardSummary {
            state: analysis.state.clone(),
            turns: analysis.turns.len(),
            remaining_candidates: analysis.final_remaining_candidates,
        },
        post_game: None,
        easy_hint: Some(hint),
        share: Some(ShareOutput {
            text: share_text,
            contains_guess_words,
        }),
    })
}

fn build_post_game_report(analysis: &BoardAnalysis) -> PostGameReport {
    let turns: Vec<TurnReview> = analysis.turns.iter().map(review_turn).collect();
    let grades = turns.iter().map(|turn| turn.grade.clone()).collect();
    let summary = game_summary(&turns);
    PostGameReport {
        grades,
        turns,
        summary,
    }
}

fn review_turn(turn: &TurnAnalysis) -> TurnReview {
    let label = turn_label(turn).to_string();
    let score = turn_score(turn);
    let grade = grade(score).to_string();
    let did_well = did_well(turn, &label);
    let missed = missed(turn);
    let summary = turn_summary(turn, &label);

    TurnReview {
        turn: turn.turn_index + 1,
        label,
        grade,
        score,
        move_type: turn.move_type,
        information: turn.information_bucket,
        constraint_discipline: turn.constraint_discipline,
        did_well,
        missed,
        summary,
    }
}

fn game_summary(turns: &[TurnReview]) -> GameSummary {
    let best_move = turns
        .iter()
        .max_by_key(|turn| (turn.score, turn.information, turn.turn))
        .expect("post-game report has at least one turn");
    let most_questionable = turns
        .iter()
        .min_by_key(|turn| {
            let forced_final_offset = if turn.move_type == MoveType::ForcedSolve {
                20
            } else {
                0
            };
            (turn.score + forced_final_offset, turn.turn)
        })
        .expect("post-game report has at least one turn");
    let biggest_information = turns
        .iter()
        .max_by_key(|turn| (turn.information, turn.score, turn.turn))
        .expect("post-game report has at least one turn");
    let best_recovery_turn = turns.windows(2).find_map(|pair| {
        let weak = pair[0].score <= 68;
        let strong_after = pair[1].score >= 83;
        (weak && strong_after).then_some(pair[1].turn)
    });
    let missed_opportunity = (most_questionable.score <= 72).then(|| {
        format!(
            "Turn {} could have focused more on {}.",
            most_questionable.turn,
            opportunity_phrase(most_questionable)
        )
    });
    let lesson = lesson(turns);

    GameSummary {
        best_move_turn: best_move.turn,
        most_questionable_turn: most_questionable.turn,
        biggest_information_gain_turn: biggest_information.turn,
        best_recovery_turn,
        missed_opportunity,
        lesson,
    }
}

fn opportunity_phrase(turn: &TurnReview) -> &'static str {
    if turn.constraint_discipline == ConstraintDiscipline::Miss {
        "using the clues already earned"
    } else if turn.move_type == MoveType::SolveAttempt
        && turn.information <= InformationBucket::Modest
    {
        "splitting the remaining pattern before guessing one answer"
    } else if turn.information <= InformationBucket::Low {
        "testing fresh information"
    } else {
        "the main uncertainty in the pattern"
    }
}

fn lesson(turns: &[TurnReview]) -> String {
    if turns
        .iter()
        .any(|turn| turn.constraint_discipline == ConstraintDiscipline::Miss)
    {
        return "Carry every green and yellow clue into the next turn before choosing a word."
            .to_string();
    }
    if turns
        .iter()
        .filter(|turn| turn.information <= InformationBucket::Low)
        .count()
        >= 2
    {
        return "When the board stalls, spend a turn separating the pattern instead of repeating the same information.".to_string();
    }
    if turns.iter().any(|turn| turn.label == "Trap Breaker") {
        return "When a word family forms, test the changing position before guessing one-by-one."
            .to_string();
    }
    if turns.iter().any(|turn| {
        turn.move_type == MoveType::SolveAttempt && turn.information <= InformationBucket::Modest
    }) {
        return "Before a direct solve, check whether several similar answers are still alive."
            .to_string();
    }
    "Keep matching the purpose of the guess to the stage of the game: broad early, answer-shaped late."
        .to_string()
}

fn build_post_game_share_text(
    analysis: &BoardAnalysis,
    report: &PostGameReport,
    hint_summary: Option<&HintShareSummary>,
) -> String {
    let result = match analysis.state {
        BoardState::Solved { turn } => format!("Wordle {turn}/6"),
        BoardState::Lost => "Wordle X/6".to_string(),
        BoardState::InProgress => "Wordle ?/6".to_string(),
    };
    let grid = analysis
        .turns
        .iter()
        .map(|turn| status_grid_row(&turn.statuses))
        .collect::<Vec<_>>()
        .join("\n");
    let grades = report.grades.join(" / ");
    let best = report.summary.best_move_turn;
    let lesson = &report.summary.lesson;

    if let Some(summary) = hint_summary {
        let hints = summary.hint_labels_used.join(", ");
        return format!(
            "{result}\n{grid}\n\nWirdle: Easy Mode + Post Game Mode\nHints: {hints}\nHighest hint: Level {}\nGrades: {grades}\nCoach: best move was turn {best}. Lesson: {lesson}\n\nwirdle.onrender.com",
            summary.highest_hint_level_used
        );
    }

    format!(
        "{result}\n{grid}\n\nWirdle: Post Game Mode\nGrades: {grades}\nCoach: best move was turn {best}. Lesson: {lesson}\n\nwirdle.onrender.com"
    )
}

fn build_easy_share_text(analysis: &BoardAnalysis, summary: &HintShareSummary) -> String {
    let result = match analysis.state {
        BoardState::Solved { turn } => format!("Wordle {turn}/6"),
        BoardState::Lost => "Wordle X/6".to_string(),
        BoardState::InProgress => "Wordle ?/6".to_string(),
    };
    let grid = analysis
        .turns
        .iter()
        .map(|turn| status_grid_row(&turn.statuses))
        .collect::<Vec<_>>()
        .join("\n");
    let hints = summary.hint_labels_used.join(", ");

    format!(
        "{result}\n{grid}\n\nWirdle: Easy Mode\nHints: {hints}\nHighest hint: Level {}\nCoach: helped me use the clues without giving it away.\n\nwirdle.onrender.com",
        summary.highest_hint_level_used
    )
}

fn choose_hint_level(
    analysis: &BoardAnalysis,
    hint_request: Option<&HintRequest>,
    session_context: Option<&SessionContext>,
) -> HintLevel {
    if let Some(request) = hint_request {
        if request.explain_current {
            if let Some(level) = request.requested_level {
                return HintLevel::from_requested(level);
            }
            if let Some(session) = session_context {
                if session.highest_hint_level_used > 0 {
                    return HintLevel::from_requested(session.highest_hint_level_used);
                }
            }
        }
        if let Some(level) = request.requested_level {
            let requested = HintLevel::from_requested(level);
            if matches!(requested, HintLevel::UsefulLetter | HintLevel::Pattern)
                && should_downgrade_spoilery_mid_hint(analysis)
            {
                return HintLevel::NextMoveStrategy;
            }
            return requested;
        }
    }

    first_hint_level(analysis)
}

fn first_hint_level(analysis: &BoardAnalysis) -> HintLevel {
    let Some(last_turn) = analysis.turns.last() else {
        return HintLevel::GentleNudge;
    };
    if last_turn.constraint_discipline == ConstraintDiscipline::Miss {
        return HintLevel::GentleNudge;
    }
    if analysis.turns.len() <= 3 && analysis.final_effective_candidates > 20.0 {
        return HintLevel::GentleNudge;
    }
    if last_turn.trap_risk != TrapRisk::Low || analysis.final_effective_candidates <= 12.0 {
        return HintLevel::NextMoveStrategy;
    }
    HintLevel::GentleNudge
}

fn should_downgrade_spoilery_mid_hint(analysis: &BoardAnalysis) -> bool {
    analysis.turns.len() <= 1 && analysis.final_effective_candidates > 80.0
}

fn build_hint_response(
    solver: &Solver,
    request: &CoachRequest,
    analysis: &BoardAnalysis,
    candidates: &[Word],
    level: HintLevel,
    explain_current: bool,
) -> HintResponse {
    let (message, base_rationale, revealed_words) = match level {
        HintLevel::GentleNudge => (
            gentle_nudge_message(analysis, &request.guesses, request.hard_mode),
            gentle_nudge_rationale(analysis),
            Vec::new(),
        ),
        HintLevel::NextMoveStrategy => (
            next_move_strategy_message(analysis, &request.guesses, candidates),
            next_move_strategy_rationale(analysis),
            Vec::new(),
        ),
        HintLevel::UsefulLetter => (
            useful_letter_message(candidates, &request.guesses),
            Some("These letters appear often in the remaining answer-shaped pool, while avoiding letters you have already tested.".to_string()),
            Vec::new(),
        ),
        HintLevel::Pattern => (
            pattern_hint(candidates, &request.guesses).unwrap_or_else(|| {
                "The structure is still too open for a clean pattern; keep the confirmed positions fixed and test a new slot.".to_string()
            }),
            Some("The pattern only uses information implied by the board and shared structure across compatible answers.".to_string()),
            Vec::new(),
        ),
        HintLevel::StrongGuessHelp => {
            let options = human_like_guess_options(solver, candidates, &request.guesses, request.hard_mode);
            let revealed_words = options.iter().take(3).map(|option| option.word).collect::<Vec<_>>();
            let message = if revealed_words.len() <= 1 {
                "This is the strongest answer-shaped direction I would consider from the current board.".to_string()
            } else {
                "Here are a few plausible directions. They all respect what you know, but they test different uncertainties.".to_string()
            };
            let rationale = options.first().map(|option| {
                format!(
                    "The leading option is still answer-shaped and has a {:.1}-bit information profile for this board. {}",
                    option.information.entropy_bits,
                    option.explanation
                )
            });
            (message, rationale, revealed_words)
        }
        HintLevel::AnswerReveal => (
            "The board now points to one compatible answer.".to_string(),
            Some("Wirdle can reveal this only because the entered feedback leaves exactly one compatible candidate.".to_string()),
            candidates.first().copied().into_iter().collect(),
        ),
    };
    let rationale = if explain_current {
        Some(expanded_rationale(
            level,
            analysis,
            base_rationale.as_deref(),
        ))
    } else {
        base_rationale
    };
    let share_summary = hint_share_summary(request.session_context.as_ref(), Some(level));

    HintResponse {
        level,
        label: level.label(),
        spoiler_risk: level.spoiler_risk(),
        message,
        rationale,
        next_action_label: level.next_action_label(),
        requires_confirmation_for_next: matches!(
            level,
            HintLevel::Pattern | HintLevel::StrongGuessHelp
        ),
        revealed_words,
        share_summary,
    }
}

fn gentle_nudge_message(
    analysis: &BoardAnalysis,
    guesses: &[GuessInput],
    hard_mode: bool,
) -> String {
    if guesses.is_empty() {
        return "Start with a broad opener that mixes common consonants and vowels.".to_string();
    }
    if analysis
        .turns
        .iter()
        .any(|turn| turn.constraint_discipline == ConstraintDiscipline::Miss)
    {
        return if hard_mode {
            "Hard Mode means the next playable guess must carry every confirmed clue forward."
                .to_string()
        } else {
            "Before choosing the next word, check that every green and yellow clue is still being used.".to_string()
        };
    }
    let yellow_letters = known_yellow_letters(guesses);
    if yellow_letters.len() > 1 {
        let clues = yellow_letters
            .iter()
            .take(3)
            .map(|yellow| {
                format!(
                    "{} away from {}",
                    letter_name(yellow.letter),
                    blocked_positions_phrase(&yellow.blocked_positions)
                )
            })
            .collect::<Vec<_>>();
        return format!("Place the yellow clues: {}.", clues.join("; "));
    }
    if let Some(yellow) = yellow_letters.first() {
        return format!(
            "Focus on moving the yellow {} away from {}.",
            letter_name(yellow.letter),
            blocked_positions_phrase(&yellow.blocked_positions)
        );
    }
    if let Some((idx, letter)) = known_green_positions(guesses).first() {
        return format!(
            "Keep the {} green {} fixed while you learn something new.",
            position_name(*idx),
            letter_name(*letter)
        );
    }
    if guessed_vowel_count(guesses) >= 4 {
        return "Most vowel information is already on the board. Common consonants matter more now."
            .to_string();
    }
    if analysis
        .turns
        .last()
        .is_some_and(|turn| turn.trap_risk != TrapRisk::Low)
    {
        return "This looks like a pattern trap; avoid guessing similar answers one by one."
            .to_string();
    }
    "This is a narrowing turn: make the next guess explain more of the board.".to_string()
}

fn gentle_nudge_rationale(analysis: &BoardAnalysis) -> Option<String> {
    let turns = analysis.turns.len();
    if turns == 0 {
        return Some("With no colors entered yet, broad coverage is more useful than committing to a narrow answer shape.".to_string());
    }
    Some(format!(
        "After {turns} turn{}, there are still {} compatible answers, so the next move should be guided by the clues already earned.",
        if turns == 1 { "" } else { "s" },
        analysis.final_remaining_candidates
    ))
}

fn next_move_strategy_message(
    analysis: &BoardAnalysis,
    guesses: &[GuessInput],
    candidates: &[Word],
) -> String {
    if guesses.is_empty() {
        return "Choose an opener that tests several common letters without repeating a letter."
            .to_string();
    }
    if analysis
        .turns
        .last()
        .is_some_and(|turn| turn.trap_risk == TrapRisk::High)
    {
        return "A pattern-splitting guess is more useful than trying similar answers one at a time."
            .to_string();
    }
    if analysis.final_effective_candidates <= 3.0 || analysis.turns.len() >= 5 {
        return "With the board this narrow, prefer a plausible answer that respects every clue."
            .to_string();
    }
    if let Some(yellow) = known_yellow_letters(guesses).first() {
        return format!(
            "Try a guess that moves the yellow {} into a new position while testing fresh consonants.",
            letter_name(yellow.letter)
        );
    }
    if let Some((idx, letter)) = known_green_positions(guesses).first() {
        return format!(
            "Look for a plausible answer that keeps the {} green {} and tests new letters.",
            position_name(*idx),
            letter_name(*letter)
        );
    }
    if let Some(idx) = important_unknown_positions(candidates, guesses).first() {
        return format!(
            "The biggest uncertainty is the {} position; make the next guess teach you about that slot.",
            position_name(*idx)
        );
    }
    "Look for a guess that stays answer-shaped while testing letters you have not learned about yet."
        .to_string()
}

fn next_move_strategy_rationale(analysis: &BoardAnalysis) -> Option<String> {
    if analysis.final_effective_candidates <= 12.0 {
        return Some("The candidate pool is small enough that answer-shaped guesses matter more than pure information probes.".to_string());
    }
    if analysis
        .turns
        .last()
        .is_some_and(|turn| turn.trap_risk != TrapRisk::Low)
    {
        return Some("Several compatible answers likely share a tight structure, so separating the changing position is valuable.".to_string());
    }
    Some(
        "This keeps the hint about the purpose of the move, without naming the move itself."
            .to_string(),
    )
}

fn useful_letter_message(candidates: &[Word], guesses: &[GuessInput]) -> String {
    let letters = useful_remaining_letters(candidates, guesses);
    if letters.is_empty() {
        return "The most useful new information is likely from an untested common consonant."
            .to_string();
    }
    format!("Testing {} would be useful.", letter_list(&letters))
}

fn expanded_rationale(
    level: HintLevel,
    analysis: &BoardAnalysis,
    base_rationale: Option<&str>,
) -> String {
    let base = base_rationale.unwrap_or("This hint follows directly from the board state.");
    match level {
        HintLevel::GentleNudge | HintLevel::NextMoveStrategy => format!(
            "{base} The board has {} compatible answer{} after {} entered turn{}, so the goal is to reduce uncertainty without giving away an answer.",
            analysis.final_remaining_candidates,
            if analysis.final_remaining_candidates == 1 {
                ""
            } else {
                "s"
            },
            analysis.turns.len(),
            if analysis.turns.len() == 1 { "" } else { "s" }
        ),
        HintLevel::UsefulLetter => format!(
            "{base} Letter hints are stronger because they introduce information that may not already be visible on your board."
        ),
        HintLevel::Pattern => format!(
            "{base} Pattern hints are intentionally spoilery, so this keeps at least one unresolved slot hidden."
        ),
        HintLevel::StrongGuessHelp => {
            format!(
                "{base} These words are shown only after confirmation because they can materially reduce the puzzle."
            )
        }
        HintLevel::AnswerReveal => {
            format!("{base} No ranking model is being treated as secret answer knowledge.")
        }
    }
}

fn hint_share_summary(
    session_context: Option<&SessionContext>,
    current_level: Option<HintLevel>,
) -> HintShareSummary {
    let mut levels = session_context
        .map(|session| session.hint_levels_used.clone())
        .unwrap_or_default();
    if let Some(session) = session_context {
        if levels.is_empty() && session.highest_hint_level_used > 0 {
            levels.push(session.highest_hint_level_used);
        }
    }
    if let Some(level) = current_level {
        levels.push(level.as_u8());
    }
    levels.retain(|level| (1..=6).contains(level));
    levels.sort_unstable();
    levels.dedup();
    let highest = levels
        .iter()
        .copied()
        .max()
        .unwrap_or(0)
        .max(session_context.map_or(0, |session| session.highest_hint_level_used));
    let hint_labels_used = levels
        .iter()
        .copied()
        .map(|level| HintLevel::from_requested(level).label().to_string())
        .collect();

    HintShareSummary {
        highest_hint_level_used: highest,
        hint_labels_used,
    }
}

fn current_candidates(solver: &Solver, guesses: &[GuessInput]) -> Vec<Word> {
    filter_candidates(&solver.lexicon().allowed_guesses, guesses)
}

fn known_green_positions(guesses: &[GuessInput]) -> Vec<(usize, u8)> {
    let mut positions = Vec::new();
    for guess in guesses {
        for (idx, status) in guess.statuses.iter().enumerate() {
            if *status == LetterStatus::Correct {
                let value = (idx, guess.word.0[idx]);
                if !positions.contains(&value) {
                    positions.push(value);
                }
            }
        }
    }
    positions.sort_unstable();
    positions
}

fn known_yellow_letters(guesses: &[GuessInput]) -> Vec<KnownYellow> {
    let mut blocked: HashMap<u8, Vec<usize>> = HashMap::new();
    for guess in guesses {
        for (idx, status) in guess.statuses.iter().enumerate() {
            if *status == LetterStatus::Present {
                let entry = blocked.entry(guess.word.0[idx]).or_default();
                if !entry.contains(&idx) {
                    entry.push(idx);
                }
            }
        }
    }
    let mut yellows = blocked
        .into_iter()
        .map(|(letter, mut blocked_positions)| {
            blocked_positions.sort_unstable();
            KnownYellow {
                letter,
                blocked_positions,
            }
        })
        .collect::<Vec<_>>();
    yellows.sort_by_key(|yellow| yellow.letter);
    yellows
}

fn known_absent_letters(guesses: &[GuessInput]) -> Vec<u8> {
    let mut confirmed = [false; 26];
    let mut absent = [false; 26];
    for guess in guesses {
        for (idx, status) in guess.statuses.iter().enumerate() {
            let letter = usize::from(guess.word.0[idx] - b'a');
            match status {
                LetterStatus::Correct | LetterStatus::Present => confirmed[letter] = true,
                LetterStatus::Absent => absent[letter] = true,
            }
        }
    }
    absent
        .iter()
        .enumerate()
        .filter_map(|(idx, value)| (*value && !confirmed[idx]).then_some(b'a' + idx as u8))
        .collect()
}

fn important_unknown_positions(candidates: &[Word], guesses: &[GuessInput]) -> Vec<usize> {
    let greens = known_green_positions(guesses);
    let mut positions = (0..5)
        .filter(|idx| !greens.iter().any(|(green_idx, _)| green_idx == idx))
        .map(|idx| {
            let mut seen = [false; 26];
            for candidate in candidates {
                seen[usize::from(candidate.0[idx] - b'a')] = true;
            }
            let count = seen.iter().filter(|value| **value).count();
            (idx, count)
        })
        .collect::<Vec<_>>();
    positions.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    positions.into_iter().map(|(idx, _)| idx).collect()
}

fn useful_remaining_letters(candidates: &[Word], guesses: &[GuessInput]) -> Vec<u8> {
    let mut guessed = [false; 26];
    for guess in guesses {
        for letter in guess.word.letters() {
            guessed[letter] = true;
        }
    }
    for letter in known_absent_letters(guesses) {
        guessed[usize::from(letter - b'a')] = true;
    }

    let mut counts = [0usize; 26];
    for candidate in candidates {
        let mut seen_in_word = [false; 26];
        for letter in candidate.letters() {
            if !guessed[letter] {
                seen_in_word[letter] = true;
            }
        }
        for (idx, seen) in seen_in_word.iter().enumerate() {
            if *seen {
                counts[idx] += 1;
            }
        }
    }
    let minimum = if candidates.len() <= 8 { 1 } else { 2 };
    let mut letters = counts
        .iter()
        .enumerate()
        .filter_map(|(idx, count)| (*count >= minimum).then_some((b'a' + idx as u8, *count)))
        .collect::<Vec<_>>();
    letters.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    letters
        .into_iter()
        .take(3)
        .map(|(letter, _)| letter)
        .collect()
}

fn pattern_hint(candidates: &[Word], guesses: &[GuessInput]) -> Option<String> {
    if candidates.is_empty() {
        return None;
    }

    let mut pattern = [None; 5];
    for (idx, letter) in known_green_positions(guesses) {
        pattern[idx] = Some(letter);
    }
    let green_count = pattern.iter().filter(|letter| letter.is_some()).count();
    let mut shared_positions = (0..5)
        .filter(|idx| pattern[*idx].is_none())
        .filter_map(|idx| {
            let first = candidates[0].0[idx];
            let shared = candidates.iter().all(|word| word.0[idx] == first);
            shared.then_some((idx, first))
        })
        .collect::<Vec<_>>();
    shared_positions.sort_unstable();
    if green_count + shared_positions.len() < 5 {
        if let Some((idx, letter)) = shared_positions.first().copied() {
            pattern[idx] = Some(letter);
        }
    }

    let pattern_text = format_pattern(&pattern);
    if pattern.iter().all(|letter| letter.is_some()) {
        return Some(
            "The structure is fully determined; use answer reveal if you want Wirdle to name it."
                .to_string(),
        );
    }

    if let Some((start, letters)) = common_edge_pattern(candidates) {
        if letters.len() >= 2 {
            let edge = if start == 0 { "opening" } else { "ending" };
            return Some(format!(
                "A useful answer-shaped pattern is `{pattern_text}`. The {edge} is doing a lot of work."
            ));
        }
    }

    Some(format!(
        "A useful answer-shaped pattern is `{pattern_text}`."
    ))
}

fn common_edge_pattern(candidates: &[Word]) -> Option<(usize, Vec<u8>)> {
    for width in (2..=4).rev() {
        for start in [0, 5 - width] {
            let first = &candidates.first()?.0[start..start + width];
            let same = candidates
                .iter()
                .filter(|word| &word.0[start..start + width] == first)
                .count();
            if same >= 2 && same * 2 >= candidates.len() {
                return Some((start, first.to_vec()));
            }
        }
    }
    None
}

/// How many of the most likely candidates get the expensive per-guess
/// evaluation.
///
/// Callers surface only the top few options, and `human_score` is dominated by
/// the likely-answer score, so the leaders always come from the head of that
/// ranking. Evaluating the whole pool instead is O(n^2) over the accepted-guess
/// universe: ~220M feedback evaluations, measured at ~38s for one hint request
/// on an empty board. Narrow boards are unaffected — they have fewer candidates
/// than this bound.
const HUMAN_OPTION_EVALUATION_LIMIT: usize = 64;

fn human_like_guess_options(
    solver: &Solver,
    candidates: &[Word],
    guesses: &[GuessInput],
    hard_mode: bool,
) -> Vec<HintGuessOption> {
    let likely_answers = rank_likely_answers(
        candidates,
        solver.past(),
        &PastSolutionPolicy::default(),
        solver.lexicon(),
    );
    let likely_scores: HashMap<Word, f64> = likely_answers
        .iter()
        .map(|answer| (answer.word, answer.score))
        .collect();
    let considered: Vec<Word> = likely_answers
        .iter()
        .map(|answer| answer.word)
        .filter(|word| !hard_mode || is_candidate_consistent(*word, guesses))
        .take(HUMAN_OPTION_EVALUATION_LIMIT)
        .collect();
    let pool = CandidatePool::new(candidates, solver.lexicon(), &likely_answers);
    let mut options = considered
        .into_iter()
        .map(|word| {
            let stats = bucket_stats(word, &pool);
            let information =
                information_guess(word, &stats, &pool, solver.past(), solver.lexicon());
            let likely_score = likely_scores.get(&word).copied().unwrap_or(0.0);
            let mut human_score = 100;
            human_score += (likely_score * 70.0).round() as i32;
            human_score += (information.entropy_bits * 12.0).round() as i32;
            human_score -= (information.expected_remaining * 2.0).round() as i32;
            if information.used_before {
                human_score -= 25;
            }
            let explanation = if information.used_before {
                "It is compatible, but it has appeared before in the loaded answer history."
                    .to_string()
            } else if candidates.len() <= 4 {
                "The board is narrow enough that a direct answer-shaped option is reasonable."
                    .to_string()
            } else {
                "It balances answer plausibility with useful separation of the remaining pool."
                    .to_string()
            };
            HintGuessOption {
                word,
                information,
                human_score,
                explanation,
            }
        })
        .collect::<Vec<_>>();
    options.sort_by(|a, b| {
        b.human_score
            .cmp(&a.human_score)
            .then_with(|| b.information.score.total_cmp(&a.information.score))
            .then_with(|| a.word.cmp(&b.word))
    });
    options
}

fn format_pattern(pattern: &[Option<u8>; 5]) -> String {
    pattern
        .iter()
        .map(|letter| letter.map(letter_name).unwrap_or_else(|| "_".to_string()))
        .collect::<Vec<_>>()
        .join(" ")
}

fn letter_name(letter: u8) -> String {
    char::from(letter).to_ascii_uppercase().to_string()
}

fn letter_list(letters: &[u8]) -> String {
    match letters {
        [] => String::new(),
        [one] => letter_name(*one),
        [one, two] => format!("{} or {}", letter_name(*one), letter_name(*two)),
        _ => {
            let mut names = letters
                .iter()
                .map(|letter| letter_name(*letter))
                .collect::<Vec<_>>();
            let last = names.pop().expect("at least one letter");
            format!("{}, or {}", names.join(", "), last)
        }
    }
}

fn blocked_positions_phrase(positions: &[usize]) -> String {
    let names = positions
        .iter()
        .map(|idx| position_name(*idx))
        .collect::<Vec<_>>();
    match names.as_slice() {
        [] => "its known bad position".to_string(),
        [one] => format!("the {one} position"),
        [one, two] => format!("the {one} or {two} position"),
        _ => {
            let last = names.last().expect("at least one position");
            let prefix = names[..names.len() - 1].join(", ");
            format!("{prefix}, or {last} position")
        }
    }
}

fn position_name(idx: usize) -> &'static str {
    match idx {
        0 => "first",
        1 => "second",
        2 => "third",
        3 => "fourth",
        _ => "fifth",
    }
}

fn guessed_vowel_count(guesses: &[GuessInput]) -> usize {
    let mut vowels = [false; 5];
    for guess in guesses {
        for letter in guess.word.0 {
            match letter {
                b'a' => vowels[0] = true,
                b'e' => vowels[1] = true,
                b'i' => vowels[2] = true,
                b'o' => vowels[3] = true,
                b'u' => vowels[4] = true,
                _ => {}
            }
        }
    }
    vowels.iter().filter(|seen| **seen).count()
}

pub fn status_grid_row(statuses: &[LetterStatus; 5]) -> String {
    statuses
        .iter()
        .map(|status| match status {
            LetterStatus::Absent => "⬛",
            LetterStatus::Present => "🟨",
            LetterStatus::Correct => "🟩",
        })
        .collect()
}

fn information_bucket(before: usize, after: usize) -> InformationBucket {
    if before <= 1 {
        return InformationBucket::Sharp;
    }
    let reduction = 1.0 - (after as f64 / before as f64);
    if after <= 1 || reduction >= 0.85 {
        InformationBucket::Sharp
    } else if reduction >= 0.55 {
        InformationBucket::Solid
    } else if reduction >= 0.25 {
        InformationBucket::Modest
    } else {
        InformationBucket::Low
    }
}

fn move_type(
    respects_known_info: bool,
    candidates_before: usize,
    turn_index: usize,
    information_bucket: InformationBucket,
    guess: crate::word::Word,
    candidates: &[crate::word::Word],
    is_likelier: bool,
) -> MoveType {
    if !respects_known_info {
        return MoveType::ConstraintMiss;
    }
    // With the accepted-guess list as the universe, mere membership is true for
    // almost every guess, so an opener would grade as a solve attempt and Probe
    // would be unreachable. Answer-shaped means likelier, as it does in ranking.
    let is_possible_answer = candidates.contains(&guess) && is_likelier;
    if is_possible_answer && (candidates_before <= 3 || turn_index >= 4) {
        MoveType::ForcedSolve
    } else if is_possible_answer {
        MoveType::SolveAttempt
    } else if information_bucket == InformationBucket::Low && turn_index >= 4 {
        MoveType::ConstraintMiss
    } else {
        MoveType::Probe
    }
}

fn stage(turn_index: usize) -> Stage {
    match turn_index {
        0 => Stage::Opener,
        5 => Stage::Final,
        3 | 4 => Stage::Endgame,
        _ => Stage::Middle,
    }
}

fn trap_risk(candidates: &[crate::word::Word], effective: f64) -> TrapRisk {
    if effective <= 4.0 {
        return TrapRisk::High;
    }
    if effective <= 10.0 && tight_family(candidates) {
        return TrapRisk::High;
    }
    if effective <= 16.0 {
        return TrapRisk::Moderate;
    }
    TrapRisk::Low
}

fn tight_family(candidates: &[crate::word::Word]) -> bool {
    (0..5).any(|skip| {
        let first = candidates.first().map(|word| {
            word.0
                .iter()
                .enumerate()
                .filter_map(|(idx, letter)| (idx != skip).then_some(*letter))
                .collect::<Vec<_>>()
        });
        first.is_some_and(|pattern| {
            candidates.iter().all(|word| {
                word.0
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, letter)| (idx != skip).then_some(*letter))
                    .collect::<Vec<_>>()
                    == pattern
            })
        })
    })
}

fn duplicate_letter_note(guess: &GuessInput) -> DuplicateLetterNote {
    let mut counts = [0u8; 26];
    for letter in guess.word.letters() {
        counts[letter] += 1;
    }
    let repeated = counts.iter().any(|count| *count > 1);
    if !repeated {
        return DuplicateLetterNote::None;
    }
    let useful = guess
        .word
        .letters()
        .iter()
        .enumerate()
        .any(|(idx, letter)| counts[*letter] > 1 && guess.statuses[idx] != LetterStatus::Absent);
    if useful {
        DuplicateLetterNote::Useful
    } else {
        DuplicateLetterNote::Risky
    }
}

fn constraint_note(
    guess: &GuessInput,
    prior: &[GuessInput],
    hard_mode: bool,
    respects_known_info: bool,
) -> Option<String> {
    if respects_known_info {
        return None;
    }

    let mut green_positions = Vec::new();
    let mut yellow_positions = Vec::new();
    let mut confirmed_counts = [0u8; 26];
    let mut absent_letters = [false; 26];
    let mut seen_confirmed = [false; 26];

    for prior_guess in prior {
        for (idx, status) in prior_guess.statuses.iter().enumerate() {
            let letter = usize::from(prior_guess.word.0[idx] - b'a');
            match status {
                LetterStatus::Correct => {
                    green_positions.push((idx, prior_guess.word.0[idx]));
                    confirmed_counts[letter] = confirmed_counts[letter].max(1);
                    seen_confirmed[letter] = true;
                }
                LetterStatus::Present => {
                    yellow_positions.push((idx, prior_guess.word.0[idx]));
                    confirmed_counts[letter] = confirmed_counts[letter].max(1);
                    seen_confirmed[letter] = true;
                }
                LetterStatus::Absent => {
                    if !seen_confirmed[letter] {
                        absent_letters[letter] = true;
                    }
                }
            }
        }
    }

    if green_positions
        .iter()
        .any(|(idx, letter)| guess.word.0[*idx] != *letter)
    {
        return Some("It moved away from a confirmed green position.".to_string());
    }
    if yellow_positions
        .iter()
        .any(|(idx, letter)| guess.word.0[*idx] == *letter)
    {
        return Some(
            "It left a known yellow letter in a place that had already been ruled out.".to_string(),
        );
    }
    let guess_counts = guess
        .word
        .letters()
        .iter()
        .fold([0u8; 26], |mut counts, letter| {
            counts[*letter] += 1;
            counts
        });
    if confirmed_counts
        .iter()
        .enumerate()
        .any(|(idx, count)| *count > 0 && guess_counts[idx] < *count)
    {
        return Some(
            "It dropped a letter that earlier feedback had already confirmed.".to_string(),
        );
    }
    if guess
        .word
        .letters()
        .iter()
        .any(|letter| absent_letters[*letter])
    {
        return Some(
            "It reused a gray letter without duplicate-letter evidence to justify it.".to_string(),
        );
    }
    if hard_mode {
        Some("It would not be playable under the known hard-mode constraints.".to_string())
    } else {
        Some("It did not line up with all of the information already on the board.".to_string())
    }
}

fn turn_label(turn: &TurnAnalysis) -> &'static str {
    if turn.constraint_discipline == ConstraintDiscipline::Miss {
        return "Constraint Miss";
    }
    if turn.stage == Stage::Opener {
        return if opener_is_vowel_heavy(turn) {
            "Vowel-Heavy Opener"
        } else {
            "Balanced Opener"
        };
    }
    if turn.move_type == MoveType::ForcedSolve || turn.move_type == MoveType::SolveAttempt {
        if turn.candidates_before > 8 && turn.information_bucket <= InformationBucket::Modest {
            return "Risky Direct Solve";
        }
        return "Candidate Solve";
    }
    if turn.trap_risk != TrapRisk::Low && turn.information_bucket >= InformationBucket::Solid {
        return "Trap Breaker";
    }
    if turn.duplicate_letter_note == DuplicateLetterNote::Useful {
        return "Duplicate-Letter Test";
    }
    if turn.information_bucket == InformationBucket::Low {
        return "Low-Information Repeat";
    }
    if turn.information_bucket >= InformationBucket::Solid {
        "Pattern Splitter"
    } else {
        "Constraint Builder"
    }
}

fn opener_is_vowel_heavy(turn: &TurnAnalysis) -> bool {
    turn.vowel_count >= 3
}

fn vowel_count(word: crate::word::Word) -> usize {
    word.0
        .iter()
        .filter(|letter| matches!(letter, b'a' | b'e' | b'i' | b'o' | b'u'))
        .count()
}

fn turn_score(turn: &TurnAnalysis) -> i32 {
    let mut score = 78;
    match turn.constraint_discipline {
        ConstraintDiscipline::Clean => score += 4,
        ConstraintDiscipline::Miss => score -= 30,
    }
    match turn.information_bucket {
        InformationBucket::Sharp => score += 14,
        InformationBucket::Solid => score += 7,
        InformationBucket::Modest => score -= 4,
        InformationBucket::Low => score -= 18,
    }
    match turn.move_type {
        MoveType::ForcedSolve => score += 8,
        MoveType::SolveAttempt => {
            if turn.candidates_before > 8 && turn.stage != Stage::Endgame {
                score -= 8;
            } else {
                score += 2;
            }
        }
        MoveType::Probe => {
            if turn.stage == Stage::Endgame || turn.stage == Stage::Final {
                score -= 5;
            }
        }
        MoveType::ConstraintMiss => score -= 8,
    }
    match turn.duplicate_letter_note {
        DuplicateLetterNote::Useful => score += 3,
        DuplicateLetterNote::Risky => score -= 5,
        DuplicateLetterNote::None => {}
    }
    if turn.trap_risk != TrapRisk::Low && turn.information_bucket >= InformationBucket::Solid {
        score += 5;
    }
    if turn.solved_on_turn {
        score += 8;
    }
    score.clamp(0, 100)
}

fn grade(score: i32) -> &'static str {
    match score {
        93..=100 => "A",
        88..=92 => "A-",
        83..=87 => "B+",
        78..=82 => "B",
        73..=77 => "B-",
        68..=72 => "C+",
        62..=67 => "C",
        56..=61 => "C-",
        48..=55 => "D",
        _ => "F",
    }
}

fn did_well(turn: &TurnAnalysis, label: &str) -> String {
    if turn.solved_on_turn {
        return "It finished the puzzle while respecting the board state.".to_string();
    }
    match label {
        "Balanced Opener" => "It started with a broad mix of useful letters.".to_string(),
        "Vowel-Heavy Opener" => "It quickly checked vowel shape for the puzzle.".to_string(),
        "Trap Breaker" => {
            "It attacked a tight word family instead of guessing one option at a time.".to_string()
        }
        "Duplicate-Letter Test" => {
            "It used a repeat to check whether duplicate letters mattered.".to_string()
        }
        "Pattern Splitter" => "It separated the remaining pattern sharply.".to_string(),
        "Constraint Builder" => {
            "It kept the known clues in play while adding some new information.".to_string()
        }
        "Candidate Solve" => "It was a plausible answer-shaped move for the board.".to_string(),
        "Risky Direct Solve" => "It gave itself a chance to solve immediately.".to_string(),
        "Low-Information Repeat" => "It still preserved the basic board constraints.".to_string(),
        "Constraint Miss" => "It may have been intended as a probe.".to_string(),
        _ => "It made progress on the board.".to_string(),
    }
}

fn missed(turn: &TurnAnalysis) -> String {
    if let Some(note) = &turn.constraint_note {
        return note.clone();
    }
    if turn.information_bucket == InformationBucket::Low {
        return "It repeated too much known information and left the answer pool broad."
            .to_string();
    }
    if turn.move_type == MoveType::SolveAttempt
        && turn.candidates_before > 8
        && turn.information_bucket <= InformationBucket::Modest
    {
        return "It guessed into a still-wide pool where a splitter would have taught more."
            .to_string();
    }
    if turn.duplicate_letter_note == DuplicateLetterNote::Risky {
        return "The duplicate letter did not earn enough new information for that stage."
            .to_string();
    }
    if turn.stage == Stage::Endgame && turn.move_type == MoveType::Probe {
        return "Late probes are costly unless they split the main remaining pattern.".to_string();
    }
    "No major strategic miss on this turn.".to_string()
}

fn turn_summary(turn: &TurnAnalysis, label: &str) -> String {
    match turn.information_bucket {
        InformationBucket::Sharp if turn.constraint_discipline == ConstraintDiscipline::Miss => {
            format!("{label}: this found information, but by stepping away from known clues.")
        }
        InformationBucket::Sharp => format!("{label}: this narrowed the puzzle sharply."),
        InformationBucket::Solid => format!("{label}: this was a useful narrowing move."),
        InformationBucket::Modest => {
            format!("{label}: this helped, but left similar answers alive.")
        }
        InformationBucket::Low => format!("{label}: this did not change the board enough."),
    }
}

fn board_inconsistent() -> CoachError {
    CoachError::new(
        "board_inconsistent",
        "This board does not match any possible Wordle answer. Check your tile colors and update the board before getting hints or analysis.",
    )
}

pub fn coach_response_json(response: &CoachResponse) -> String {
    let post_game = response
        .post_game
        .as_ref()
        .map(post_game_json)
        .unwrap_or_else(|| "null".to_string());
    let easy_hint = response
        .easy_hint
        .as_ref()
        .map(hint_response_json)
        .unwrap_or_else(|| "null".to_string());
    let share = response
        .share
        .as_ref()
        .map(share_json)
        .unwrap_or_else(|| "null".to_string());
    format!(
        "{{\"intent\":\"{}\",\"board\":{},\"post_game\":{},\"easy_hint\":{},\"share\":{}}}",
        response.intent.as_str(),
        board_json(&response.board),
        post_game,
        easy_hint,
        share
    )
}

fn board_json(board: &BoardSummary) -> String {
    let state = match board.state {
        BoardState::InProgress => "in_progress",
        BoardState::Solved { .. } => "solved",
        BoardState::Lost => "lost",
    };
    format!(
        "{{\"state\":\"{}\",\"turns\":{},\"remaining_candidates\":{}}}",
        state, board.turns, board.remaining_candidates
    )
}

fn post_game_json(report: &PostGameReport) -> String {
    let grades = report
        .grades
        .iter()
        .map(|grade| format!("\"{}\"", escape_json(grade)))
        .collect::<Vec<_>>()
        .join(",");
    let turns = report
        .turns
        .iter()
        .map(turn_review_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"grades\":[{}],\"turns\":[{}],\"summary\":{}}}",
        grades,
        turns,
        game_summary_json(&report.summary)
    )
}

fn turn_review_json(turn: &TurnReview) -> String {
    format!(
        "{{\"turn\":{},\"label\":\"{}\",\"grade\":\"{}\",\"move_type\":\"{}\",\"information\":\"{}\",\"constraint_discipline\":\"{}\",\"did_well\":\"{}\",\"missed\":\"{}\",\"summary\":\"{}\"}}",
        turn.turn,
        escape_json(&turn.label),
        escape_json(&turn.grade),
        turn.move_type.as_str(),
        turn.information.as_str(),
        turn.constraint_discipline.as_str(),
        escape_json(&turn.did_well),
        escape_json(&turn.missed),
        escape_json(&turn.summary)
    )
}

fn game_summary_json(summary: &GameSummary) -> String {
    let recovery = summary
        .best_recovery_turn
        .map(|turn| turn.to_string())
        .unwrap_or_else(|| "null".to_string());
    let missed = summary
        .missed_opportunity
        .as_ref()
        .map(|value| format!("\"{}\"", escape_json(value)))
        .unwrap_or_else(|| "null".to_string());
    format!(
        "{{\"best_move_turn\":{},\"most_questionable_turn\":{},\"biggest_information_gain_turn\":{},\"best_recovery_turn\":{},\"missed_opportunity\":{},\"lesson\":\"{}\"}}",
        summary.best_move_turn,
        summary.most_questionable_turn,
        summary.biggest_information_gain_turn,
        recovery,
        missed,
        escape_json(&summary.lesson)
    )
}

fn share_json(share: &ShareOutput) -> String {
    format!(
        "{{\"text\":\"{}\",\"contains_guess_words\":{}}}",
        escape_json(&share.text),
        share.contains_guess_words
    )
}

fn hint_response_json(hint: &HintResponse) -> String {
    let rationale = hint
        .rationale
        .as_ref()
        .map(|value| format!("\"{}\"", escape_json(value)))
        .unwrap_or_else(|| "null".to_string());
    let next_action_label = hint
        .next_action_label
        .map(|value| format!("\"{}\"", escape_json(value)))
        .unwrap_or_else(|| "null".to_string());
    let revealed_words = hint
        .revealed_words
        .iter()
        .map(|word| format!("\"{}\"", word))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"level\":{},\"label\":\"{}\",\"spoiler_risk\":\"{}\",\"message\":\"{}\",\"rationale\":{},\"next_action_label\":{},\"requires_confirmation_for_next\":{},\"revealed_words\":[{}],\"share_summary\":{}}}",
        hint.level.as_u8(),
        hint.label,
        hint.spoiler_risk.as_str(),
        escape_json(&hint.message),
        rationale,
        next_action_label,
        hint.requires_confirmation_for_next,
        revealed_words,
        hint_share_summary_json(&hint.share_summary)
    )
}

fn hint_share_summary_json(summary: &HintShareSummary) -> String {
    let labels = summary
        .hint_labels_used
        .iter()
        .map(|label| format!("\"{}\"", escape_json(label)))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"highest_hint_level_used\":{},\"hint_labels_used\":[{}]}}",
        summary.highest_hint_level_used, labels
    )
}

fn escape_json(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
