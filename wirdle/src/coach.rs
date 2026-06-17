use crate::feedback::LetterStatus;
use crate::filter::{GuessInput, filter_candidates, is_candidate_consistent};
use crate::rank::evaluate_information_guess;
use crate::rank::{InformationGuess, rank_likely_answers};
use crate::solver::{PastSolutionPolicy, Solver};

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
        CoachIntent::EasyHint => Err(CoachError::new(
            "unsupported_intent",
            "Easy Mode hints are not implemented in Milestone 1.",
        )),
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
    let mut final_remaining_candidates = solver.lexicon().candidate_solutions.len();

    for (idx, guess) in guesses.iter().enumerate() {
        if solved_turn.is_some() {
            return Err(CoachError::new(
                "invalid_request",
                "Rows after a solved Wordle row are not valid game data.",
            ));
        }

        let candidates_before = filter_candidates(&solver.lexicon().candidate_solutions, &prior);
        if candidates_before.is_empty() {
            return Err(board_inconsistent());
        }

        let likely_answers = rank_likely_answers(
            &candidates_before,
            solver.past(),
            &PastSolutionPolicy::default(),
            &solver.lexicon().overrides,
        );
        let information = evaluate_information_guess(
            guess.word,
            &candidates_before,
            &likely_answers,
            solver.past(),
        );
        let mut with_current = prior.clone();
        with_current.push(guess.clone());
        let candidates_after =
            filter_candidates(&solver.lexicon().candidate_solutions, &with_current);
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
            ),
            stage: stage(idx),
            trap_risk: trap_risk(&candidates_before),
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
    let share_text = build_share_text(&analysis, &report);
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

fn build_share_text(analysis: &BoardAnalysis, report: &PostGameReport) -> String {
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

    format!(
        "{result}\n{grid}\n\nWirdle: Post Game Mode\nGrades: {grades}\nCoach: best move was turn {best}. Lesson: {lesson}\n\nwirdle.onrender.com"
    )
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
) -> MoveType {
    if !respects_known_info {
        return MoveType::ConstraintMiss;
    }
    let is_possible_answer = candidates.contains(&guess);
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

fn trap_risk(candidates: &[crate::word::Word]) -> TrapRisk {
    if candidates.len() <= 4 {
        return TrapRisk::High;
    }
    if candidates.len() <= 10 && tight_family(candidates) {
        return TrapRisk::High;
    }
    if candidates.len() <= 16 {
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
    let share = response
        .share
        .as_ref()
        .map(share_json)
        .unwrap_or_else(|| "null".to_string());
    format!(
        "{{\"intent\":\"{}\",\"board\":{},\"post_game\":{},\"share\":{}}}",
        response.intent.as_str(),
        board_json(&response.board),
        post_game,
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

fn escape_json(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
