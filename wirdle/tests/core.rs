use wordle_api::coach::{
    CoachIntent, CoachRequest, HintLevel, HintRequest, SessionContext, coach, status_grid_row,
};
use wordle_api::feedback::{LetterStatus::*, pattern_from_statuses, statuses_from_pattern};
use wordle_api::filter::GuessInput;
use wordle_api::rank::{evaluate_information_guess, rank_information_guesses, rank_likely_answers};
use wordle_api::server::{handle_http_request, parse_coach_request, parse_solve_request};
use wordle_api::solver::{
    PastSolutionPolicy, SolveMode, SolveRequest, entry, off_list_backtest_cases,
};
use wordle_api::{
    EditorialOverrides, Lexicon, PastSolutionIndex, Solver, Word, evaluate_feedback,
    filter_candidates, known_backtest_cases, run_backtest,
};

fn word(input: &str) -> Word {
    Word::parse(input).unwrap()
}

const FIXTURE_DATA_DIR: &str = "wordle-data";

fn fixture_solver() -> Solver {
    Solver::load(FIXTURE_DATA_DIR).unwrap()
}

fn solve_request(mode: SolveMode, limit: usize) -> SolveRequest {
    SolveRequest {
        guesses: vec![],
        mode,
        hard_mode: false,
        limit,
        past_solution_policy: PastSolutionPolicy::default(),
    }
}

fn played_game(answer: &str, guesses: &[&str]) -> Vec<GuessInput> {
    let answer = word(answer);
    guesses
        .iter()
        .map(|guess| {
            let guess = word(guess);
            GuessInput::new(
                guess,
                statuses_from_pattern(evaluate_feedback(guess, answer)),
            )
        })
        .collect()
}

fn coach_request(intent: CoachIntent, guesses: Vec<GuessInput>) -> CoachRequest {
    CoachRequest {
        intent,
        guesses,
        hard_mode: false,
        hint_request: None,
        session_context: None,
    }
}

fn easy_hint_request(
    guesses: Vec<GuessInput>,
    requested_level: Option<u8>,
    confirmed_spoiler: bool,
) -> CoachRequest {
    CoachRequest {
        intent: CoachIntent::EasyHint,
        guesses,
        hard_mode: false,
        hint_request: Some(HintRequest {
            requested_level,
            explain_current: false,
            confirmed_spoiler,
        }),
        session_context: None,
    }
}

#[test]
fn feedback_handles_duplicate_letters_in_guess() {
    let pattern = evaluate_feedback(word("eerie"), word("couch"));
    assert_eq!(
        pattern,
        pattern_from_statuses([Absent, Absent, Absent, Absent, Absent])
    );

    let pattern = evaluate_feedback(word("eerie"), word("stale"));
    assert_eq!(
        pattern,
        pattern_from_statuses([Absent, Absent, Absent, Absent, Correct])
    );
}

#[test]
fn feedback_handles_duplicate_letters_in_answer() {
    let pattern = evaluate_feedback(word("stuck"), word("stuff"));
    assert_eq!(
        pattern,
        pattern_from_statuses([Correct, Correct, Correct, Absent, Absent])
    );
}

#[test]
fn yellow_cannot_reuse_consumed_answer_letter() {
    let pattern = evaluate_feedback(word("banal"), word("clang"));
    assert_eq!(
        pattern,
        pattern_from_statuses([Absent, Present, Present, Absent, Present])
    );
}

#[test]
fn candidate_filter_uses_exact_feedback_patterns() {
    let candidates = vec![word("stuck"), word("stuff"), word("study")];
    let observed = vec![GuessInput::new(
        word("stuck"),
        [Correct, Correct, Correct, Absent, Absent],
    )];
    assert_eq!(
        filter_candidates(&candidates, &observed),
        vec![word("stuff"), word("study")]
    );
}

#[test]
fn past_solution_penalty_downweights_without_eliminating() {
    let lexicon = Lexicon {
        allowed_guesses: vec![word("pride"), word("brain")],
        likelier_solutions: [word("pride"), word("brain")].into_iter().collect(),
        overrides: EditorialOverrides::default(),
    };
    let past = PastSolutionIndex::from_entries(vec![entry("2026-05-20", 1, "pride")]);
    let solver = Solver::new(lexicon, past);
    let response = solver
        .solve(&SolveRequest {
            guesses: vec![],
            mode: SolveMode::LikelyAnswer,
            hard_mode: false,
            limit: 10,
            past_solution_policy: PastSolutionPolicy::default(),
        })
        .unwrap();

    assert!(
        response
            .likely_answers
            .iter()
            .any(|answer| answer.word == word("pride"))
    );
    assert!(response.likely_answers[0].word != word("pride"));
}

#[test]
fn current_day_answer_is_excluded_from_as_of_past_solutions() {
    let past = PastSolutionIndex::load("wordle-data/past_solutions.json").unwrap();
    let as_of = past.as_of_before("2026-05-28");

    assert!(!as_of.was_ever_solution(word("divot")));
    assert!(as_of.was_ever_solution(word("stuff")));
}

#[test]
fn entropy_ranking_handles_one_remaining_candidate() {
    let solver = fixture_solver();
    let response = solver
        .solve(&SolveRequest {
            guesses: vec![GuessInput::new(
                word("slate"),
                [Absent, Absent, Absent, Absent, Absent],
            )],
            mode: SolveMode::MaxInformation,
            hard_mode: false,
            limit: 10,
            past_solution_policy: PastSolutionPolicy {
                enabled: false,
                ..PastSolutionPolicy::default()
            },
        })
        .unwrap();

    assert!(!response.best_information_guesses.is_empty());
    assert!(response.best_information_guesses[0].entropy_bits >= 0.0);
}

#[test]
fn no_candidates_error_is_clear() {
    let solver = fixture_solver();
    let err = solver
        .solve(&SolveRequest {
            guesses: vec![GuessInput::new(
                word("slate"),
                [Correct, Correct, Correct, Correct, Present],
            )],
            mode: SolveMode::Hybrid,
            hard_mode: false,
            limit: 10,
            past_solution_policy: PastSolutionPolicy::default(),
        })
        .unwrap_err();

    assert!(err.contains("No candidate solution matches"));
    assert!(err.contains("duplicate-letter feedback"));
}

#[test]
fn hard_mode_limits_information_guesses_to_consistent_words() {
    let solver = fixture_solver();
    let prior = vec![GuessInput::new(
        word("slate"),
        [Absent, Absent, Absent, Absent, Absent],
    )];
    let response = solver
        .solve(&SolveRequest {
            guesses: prior.clone(),
            mode: SolveMode::MaxInformation,
            hard_mode: true,
            limit: 20,
            past_solution_policy: PastSolutionPolicy {
                enabled: false,
                ..PastSolutionPolicy::default()
            },
        })
        .unwrap();

    assert!(
        response
            .best_information_guesses
            .iter()
            .all(|guess| wordle_api::is_candidate_consistent(guess.word, &prior))
    );
}

#[test]
fn evaluate_information_guess_matches_ranked_information_values() {
    let solver = fixture_solver();
    let candidates = vec![word("couch"), word("clang"), word("divot")];
    let policy = PastSolutionPolicy {
        enabled: false,
        ..PastSolutionPolicy::default()
    };
    let likely = rank_likely_answers(&candidates, solver.past(), &policy, solver.lexicon());
    let ranked = rank_information_guesses(
        &[word("slate")],
        &candidates,
        &likely,
        solver.past(),
        solver.lexicon(),
    );
    let evaluated = evaluate_information_guess(
        word("slate"),
        &candidates,
        &likely,
        solver.past(),
        solver.lexicon(),
    );

    assert_eq!(ranked.len(), 1);
    assert_eq!(ranked[0].word, evaluated.word);
    assert_eq!(
        ranked[0].worst_case_remaining,
        evaluated.worst_case_remaining
    );
    assert_eq!(ranked[0].is_possible_answer, evaluated.is_possible_answer);
    assert!((ranked[0].entropy_bits - evaluated.entropy_bits).abs() < f64::EPSILON);
    assert!((ranked[0].expected_remaining - evaluated.expected_remaining).abs() < f64::EPSILON);
    assert!((ranked[0].score - evaluated.score).abs() < f64::EPSILON);
}

#[test]
fn post_game_coach_accepts_solved_board_and_omits_guess_words_from_share() {
    let solver = fixture_solver();
    let guesses = played_game("visit", &["slate", "crown", "visit"]);
    let response = coach(
        &solver,
        &CoachRequest {
            intent: CoachIntent::PostGameReview,
            guesses: guesses.clone(),
            hard_mode: false,
            hint_request: None,
            session_context: None,
        },
    )
    .unwrap();
    let report = response.post_game.as_ref().unwrap();
    let share = response.share.as_ref().unwrap();

    assert_eq!(response.board.turns, 3);
    assert_eq!(report.turns.len(), 3);
    assert_eq!(report.grades.len(), 3);
    assert!(share.text.contains("Wirdle: Post Game Mode"));
    assert!(share.text.contains("wirdle.onrender.com"));
    assert!(!share.contains_guess_words);
    for guess in guesses {
        assert!(!share.text.to_lowercase().contains(guess.word.as_str()));
    }
}

#[test]
fn post_game_coach_accepts_six_row_loss() {
    let solver = fixture_solver();
    let guesses = played_game(
        "couch",
        &["slate", "pride", "brink", "flame", "gypsy", "zonal"],
    );
    let response = coach(
        &solver,
        &CoachRequest {
            intent: CoachIntent::PostGameReview,
            guesses,
            hard_mode: false,
            hint_request: None,
            session_context: None,
        },
    )
    .unwrap();

    assert_eq!(response.board.turns, 6);
    assert!(response.share.unwrap().text.starts_with("Wordle X/6"));
}

#[test]
fn post_game_coach_rejects_incomplete_and_inconsistent_boards() {
    let solver = fixture_solver();
    let incomplete = coach(
        &solver,
        &CoachRequest {
            intent: CoachIntent::PostGameReview,
            guesses: played_game("visit", &["slate"]),
            hard_mode: false,
            hint_request: None,
            session_context: None,
        },
    )
    .unwrap_err();
    assert_eq!(incomplete.code, "board_incomplete");

    let inconsistent = coach(
        &solver,
        &CoachRequest {
            intent: CoachIntent::PostGameReview,
            guesses: vec![GuessInput::new(
                word("slate"),
                [Correct, Correct, Correct, Correct, Present],
            )],
            hard_mode: false,
            hint_request: None,
            session_context: None,
        },
    )
    .unwrap_err();
    assert_eq!(inconsistent.code, "board_inconsistent");
}

#[test]
fn post_game_coach_rejects_rows_after_solve() {
    let solver = fixture_solver();
    let guesses = played_game("visit", &["visit", "couch"]);
    let err = coach(
        &solver,
        &CoachRequest {
            intent: CoachIntent::PostGameReview,
            guesses,
            hard_mode: false,
            hint_request: None,
            session_context: None,
        },
    )
    .unwrap_err();

    assert_eq!(err.code, "invalid_request");
}

#[test]
fn easy_hint_accepts_in_progress_board_and_starts_low() {
    let solver = fixture_solver();
    let response = coach(
        &solver,
        &coach_request(
            CoachIntent::EasyHint,
            played_game("visit", &["slate", "crown"]),
        ),
    )
    .unwrap();
    let hint = response.easy_hint.as_ref().unwrap();
    let share = response.share.as_ref().unwrap();

    assert_eq!(response.intent, CoachIntent::EasyHint);
    assert!(hint.level <= HintLevel::NextMoveStrategy);
    assert!(hint.revealed_words.is_empty());
    assert!(share.text.starts_with("Wordle ?/6"));
    assert!(share.text.contains("Wirdle: Easy Mode"));
    assert!(!share.contains_guess_words);
}

#[test]
fn easy_hint_rejects_bad_or_finished_boards() {
    let solver = fixture_solver();
    let inconsistent = coach(
        &solver,
        &coach_request(
            CoachIntent::EasyHint,
            vec![GuessInput::new(
                word("slate"),
                [Correct, Correct, Correct, Correct, Present],
            )],
        ),
    )
    .unwrap_err();
    assert_eq!(inconsistent.code, "board_inconsistent");

    let rows_after_solve = coach(
        &solver,
        &coach_request(
            CoachIntent::EasyHint,
            played_game("visit", &["visit", "couch"]),
        ),
    )
    .unwrap_err();
    assert_eq!(rows_after_solve.code, "invalid_request");

    let solved = coach(
        &solver,
        &coach_request(CoachIntent::EasyHint, played_game("visit", &["visit"])),
    )
    .unwrap_err();
    assert_eq!(solved.code, "game_finished");

    let lost = coach(
        &solver,
        &coach_request(
            CoachIntent::EasyHint,
            played_game(
                "couch",
                &["slate", "pride", "brink", "flame", "gypsy", "zonal"],
            ),
        ),
    )
    .unwrap_err();
    assert_eq!(lost.code, "game_finished");
}

#[test]
fn easy_hint_escalates_and_explain_current_does_not_advance() {
    let solver = fixture_solver();
    let guesses = played_game("visit", &["slate", "crown"]);
    let stronger = CoachRequest {
        intent: CoachIntent::EasyHint,
        guesses: guesses.clone(),
        hard_mode: false,
        hint_request: Some(HintRequest {
            requested_level: Some(2),
            explain_current: false,
            confirmed_spoiler: false,
        }),
        session_context: Some(SessionContext {
            highest_hint_level_used: 1,
            hint_levels_used: vec![1],
        }),
    };
    let response = coach(&solver, &stronger).unwrap();
    let hint = response.easy_hint.as_ref().unwrap();
    assert_eq!(hint.level, HintLevel::NextMoveStrategy);
    assert_eq!(hint.share_summary.highest_hint_level_used, 2);
    assert!(
        hint.share_summary
            .hint_labels_used
            .contains(&"Gentle Nudge".to_string())
    );

    let explain = CoachRequest {
        hint_request: Some(HintRequest {
            requested_level: Some(1),
            explain_current: true,
            confirmed_spoiler: false,
        }),
        session_context: Some(SessionContext {
            highest_hint_level_used: 1,
            hint_levels_used: vec![1],
        }),
        ..stronger
    };
    let response = coach(&solver, &explain).unwrap();
    let hint = response.easy_hint.as_ref().unwrap();
    assert_eq!(hint.level, HintLevel::GentleNudge);
    assert_eq!(hint.share_summary.highest_hint_level_used, 1);
    assert!(hint.rationale.as_ref().unwrap().contains("compatible"));
}

#[test]
fn easy_hint_spoiler_levels_are_gated_and_share_safe() {
    let solver = fixture_solver();
    let guesses = played_game("visit", &["slate", "crown"]);

    let level_five =
        coach(&solver, &easy_hint_request(guesses.clone(), Some(5), false)).unwrap_err();
    assert_eq!(level_five.code, "spoiler_confirmation_required");

    let level_six =
        coach(&solver, &easy_hint_request(guesses.clone(), Some(6), false)).unwrap_err();
    assert_eq!(level_six.code, "spoiler_confirmation_required");

    let response = coach(&solver, &easy_hint_request(guesses, Some(5), true)).unwrap();
    let hint = response.easy_hint.as_ref().unwrap();
    let share = response.share.as_ref().unwrap();
    assert_eq!(hint.level, HintLevel::StrongGuessHelp);
    assert!(!hint.revealed_words.is_empty());
    assert!(hint.revealed_words.len() <= 3);
    for word in &hint.revealed_words {
        assert!(!share.text.to_lowercase().contains(word.as_str()));
    }
    assert!(!share.contains_guess_words);
}

#[test]
fn answer_reveal_requires_one_compatible_candidate() {
    let solver = fixture_solver();
    let too_many = coach(
        &solver,
        &easy_hint_request(played_game("visit", &["slate"]), Some(6), true),
    )
    .unwrap_err();
    assert_eq!(too_many.code, "answer_reveal_unavailable");

    let guesses = single_candidate_in_progress_game(&solver, "visit");
    let response = coach(&solver, &easy_hint_request(guesses, Some(6), true)).unwrap();
    let hint = response.easy_hint.as_ref().unwrap();
    assert_eq!(hint.level, HintLevel::AnswerReveal);
    assert_eq!(hint.revealed_words, vec![word("visit")]);
    assert!(!response.share.as_ref().unwrap().text.contains("visit"));
}

#[test]
fn useful_letters_and_pattern_hints_do_not_reveal_words() {
    let solver = fixture_solver();
    let guesses = played_game("couch", &["slate", "pride"]);

    let letters = coach(&solver, &easy_hint_request(guesses.clone(), Some(3), false)).unwrap();
    let letter_hint = letters.easy_hint.as_ref().unwrap();
    assert_eq!(letter_hint.level, HintLevel::UsefulLetter);
    assert!(letter_hint.revealed_words.is_empty());
    assert!(!letter_hint.message.contains("couch"));

    let pattern = coach(&solver, &easy_hint_request(guesses, Some(4), false)).unwrap();
    let pattern_hint = pattern.easy_hint.as_ref().unwrap();
    assert_eq!(pattern_hint.level, HintLevel::Pattern);
    assert!(pattern_hint.revealed_words.is_empty());
    assert!(!pattern_hint.message.contains("couch"));
}

#[test]
fn hard_mode_strong_hint_options_respect_known_constraints() {
    let solver = fixture_solver();
    let guesses = played_game("visit", &["slate", "crown"]);
    let request = CoachRequest {
        intent: CoachIntent::EasyHint,
        guesses: guesses.clone(),
        hard_mode: true,
        hint_request: Some(HintRequest {
            requested_level: Some(5),
            explain_current: false,
            confirmed_spoiler: true,
        }),
        session_context: None,
    };
    let response = coach(&solver, &request).unwrap();
    let hint = response.easy_hint.as_ref().unwrap();

    assert!(!hint.revealed_words.is_empty());
    assert!(
        hint.revealed_words
            .iter()
            .all(|word| wordle_api::is_candidate_consistent(*word, &guesses))
    );
}

#[test]
fn post_game_share_includes_easy_mode_usage() {
    let solver = fixture_solver();
    let request = CoachRequest {
        intent: CoachIntent::PostGameReview,
        guesses: played_game("visit", &["slate", "crown", "visit"]),
        hard_mode: false,
        hint_request: None,
        session_context: Some(SessionContext {
            highest_hint_level_used: 2,
            hint_levels_used: vec![1, 2],
        }),
    };
    let response = coach(&solver, &request).unwrap();
    let share = response.share.as_ref().unwrap();

    assert!(share.text.contains("Wirdle: Easy Mode + Post Game Mode"));
    assert!(share.text.contains("Highest hint: Level 2"));
    assert!(share.text.contains("Grades:"));
    assert!(!share.contains_guess_words);
}

fn single_candidate_in_progress_game(solver: &Solver, answer: &str) -> Vec<GuessInput> {
    let answer = word(answer);
    let mut observed = Vec::new();
    for _ in 0..5 {
        let candidates = filter_candidates(&solver.lexicon().allowed_guesses, &observed);
        if candidates == vec![answer] {
            return observed;
        }
        let guess = solver
            .lexicon()
            .allowed_guesses
            .iter()
            .copied()
            .filter(|candidate| *candidate != answer)
            .min_by_key(|candidate| {
                let mut trial = observed.clone();
                trial.push(GuessInput::new(
                    *candidate,
                    statuses_from_pattern(evaluate_feedback(*candidate, answer)),
                ));
                filter_candidates(&solver.lexicon().allowed_guesses, &trial).len()
            })
            .expect("candidate guess");
        observed.push(GuessInput::new(
            guess,
            statuses_from_pattern(evaluate_feedback(guess, answer)),
        ));
    }
    assert_eq!(
        filter_candidates(&solver.lexicon().allowed_guesses, &observed),
        vec![answer]
    );
    observed
}

#[test]
fn share_grid_renders_wordle_squares() {
    assert_eq!(
        status_grid_row(&[Absent, Present, Correct, Absent, Correct]),
        "⬛🟨🟩⬛🟩"
    );
}

#[test]
fn parses_and_handles_solve_http_request() {
    let body = r#"{
      "guesses": [{"word": "slate", "statuses": ["absent", "absent", "absent", "absent", "absent"]}],
      "mode": "hybrid",
      "hard_mode": false,
      "limit": 3
    }"#;
    let parsed = parse_solve_request(body).unwrap();
    assert_eq!(parsed.guesses.len(), 1);
    assert_eq!(parsed.limit, 3);

    let request = format!(
        "POST /v1/solve HTTP/1.1\r\ncontent-length: {}\r\n\r\n{}",
        body.len(),
        body
    );
    let (status, content_type, response) = handle_http_request(&request, &fixture_solver());
    assert_eq!(status, "200 OK");
    assert_eq!(content_type, "application/json");
    assert!(response.contains("\"remaining_candidates\""));
}

#[test]
fn parses_and_handles_coach_http_request() {
    let guesses = played_game("visit", &["slate", "crown", "visit"]);
    let guesses_json = guesses
        .iter()
        .map(|guess| {
            format!(
                "{{\"word\":\"{}\",\"statuses\":{}}}",
                guess.word,
                wordle_api::solver::statuses_json(&guess.statuses)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let body = format!(
        "{{\"intent\":\"post_game_review\",\"guesses\":[{}],\"hard_mode\":false}}",
        guesses_json
    );
    let parsed = parse_coach_request(&body).unwrap();
    assert_eq!(parsed.intent, CoachIntent::PostGameReview);
    assert_eq!(parsed.guesses.len(), 3);

    let request = format!(
        "POST /v1/coach HTTP/1.1\r\ncontent-length: {}\r\n\r\n{}",
        body.len(),
        body
    );
    let (status, content_type, response) = handle_http_request(&request, &fixture_solver());
    assert_eq!(status, "200 OK");
    assert_eq!(content_type, "application/json");
    assert!(response.contains("\"post_game\""));
    assert!(response.contains("wirdle.onrender.com"));
}

#[test]
fn parses_and_handles_easy_hint_http_request() {
    let body = r#"{
      "intent": "easy_hint",
      "guesses": [{"word": "slate", "statuses": ["present", "absent", "absent", "present", "absent"]}],
      "hard_mode": false,
      "hint_request": {"requested_level": 2},
      "session_context": {"highest_hint_level_used": 1, "hint_levels_used": [1]}
    }"#;
    let parsed = parse_coach_request(body).unwrap();
    assert_eq!(parsed.intent, CoachIntent::EasyHint);
    assert_eq!(parsed.hint_request.unwrap().requested_level, Some(2));
    assert_eq!(parsed.session_context.unwrap().hint_levels_used, vec![1]);

    let request = format!(
        "POST /v1/coach HTTP/1.1\r\ncontent-length: {}\r\n\r\n{}",
        body.len(),
        body
    );
    let (status, content_type, response) = handle_http_request(&request, &fixture_solver());
    assert_eq!(status, "200 OK");
    assert_eq!(content_type, "application/json");
    assert!(response.contains("\"easy_hint\""));
    assert!(response.contains("\"level\":2"));
    assert!(response.contains("Highest hint: Level 2"));
}

#[test]
fn easy_hint_http_rejects_malformed_request_and_missing_confirmation() {
    let malformed = r#"{
      "intent": "easy_hint",
      "guesses": [],
      "hint_request": {"requested_level": "strong"}
    }"#;
    let request = format!(
        "POST /v1/coach HTTP/1.1\r\ncontent-length: {}\r\n\r\n{}",
        malformed.len(),
        malformed
    );
    let (status, _, response) = handle_http_request(&request, &fixture_solver());
    assert_eq!(status, "400 Bad Request");
    assert!(response.contains("requested_level must be a number"));

    let gated = r#"{
      "intent": "easy_hint",
      "guesses": [{"word": "slate", "statuses": ["present", "absent", "absent", "present", "absent"]}],
      "hint_request": {"requested_level": 5}
    }"#;
    let request = format!(
        "POST /v1/coach HTTP/1.1\r\ncontent-length: {}\r\n\r\n{}",
        gated.len(),
        gated
    );
    let (status, _, response) = handle_http_request(&request, &fixture_solver());
    assert_eq!(status, "422 Unprocessable Entity");
    assert!(response.contains("spoiler_confirmation_required"));
}

#[test]
fn coach_http_rejects_malformed_incomplete_and_inconsistent_boards() {
    let solver = fixture_solver();
    let malformed =
        r#"{"intent":"post_game_review","guesses":[{"word":"slate","statuses":["absent"]}]}"#;
    let request = format!(
        "POST /v1/coach HTTP/1.1\r\ncontent-length: {}\r\n\r\n{}",
        malformed.len(),
        malformed
    );
    let (status, _, response) = handle_http_request(&request, &solver);
    assert_eq!(status, "400 Bad Request");
    assert!(response.contains("guess statuses must contain five values"));

    let incomplete = r#"{"intent":"post_game_review","guesses":[{"word":"slate","statuses":["absent","absent","absent","absent","absent"]}]}"#;
    let request = format!(
        "POST /v1/coach HTTP/1.1\r\ncontent-length: {}\r\n\r\n{}",
        incomplete.len(),
        incomplete
    );
    let (status, _, response) = handle_http_request(&request, &solver);
    assert_eq!(status, "422 Unprocessable Entity");
    assert!(response.contains("board_incomplete"));

    let inconsistent = r#"{"intent":"post_game_review","guesses":[{"word":"slate","statuses":["correct","correct","correct","correct","present"]}]}"#;
    let request = format!(
        "POST /v1/coach HTTP/1.1\r\ncontent-length: {}\r\n\r\n{}",
        inconsistent.len(),
        inconsistent
    );
    let (status, _, response) = handle_http_request(&request, &solver);
    assert_eq!(status, "422 Unprocessable Entity");
    assert!(response.contains("board_inconsistent"));
}

#[test]
fn serves_static_ui_as_html() {
    let (status, content_type, response) =
        handle_http_request("GET / HTTP/1.1\r\n\r\n", &fixture_solver());

    assert_eq!(status, "200 OK");
    assert_eq!(content_type, "text/html; charset=utf-8");
    assert!(response.contains("Wordle Coach"));
}

#[test]
fn serves_wirdle_icon_as_svg() {
    let (status, content_type, response) =
        handle_http_request("GET /wirdle-icon.svg HTTP/1.1\r\n\r\n", &fixture_solver());

    assert_eq!(status, "200 OK");
    assert_eq!(content_type, "image/svg+xml");
    assert!(response.contains("Wirdle icon"));
}

#[test]
fn health_includes_historical_solution_dates() {
    let past = PastSolutionIndex::load("wordle-data/past_solutions.json").unwrap();
    let (first_date, latest_date) = past.date_range().unwrap();
    let (status, content_type, response) =
        handle_http_request("GET /v1/health HTTP/1.1\r\n\r\n", &fixture_solver());

    assert_eq!(status, "200 OK");
    assert_eq!(content_type, "application/json");
    assert!(response.contains("\"past_solutions\":"));
    assert!(response.contains(&format!("\"past_solution_first_date\":\"{first_date}\"")));
    assert!(response.contains(&format!("\"past_solution_latest_date\":\"{latest_date}\"")));
}

#[test]
fn backtest_solves_fixture_and_off_list_cases() {
    let lexicon = Lexicon::load(FIXTURE_DATA_DIR).unwrap();
    let past = PastSolutionIndex::load(format!("{FIXTURE_DATA_DIR}/past_solutions.json")).unwrap();
    let cases = known_backtest_cases();
    let games = run_backtest(&lexicon, &past, &cases, 6);

    assert_eq!(games.len(), cases.len());
    assert!(games.iter().all(|game| game.solved));
    assert!(games.iter().all(|game| game.guesses.len() <= 6));
}

#[test]
fn backtest_solves_answers_absent_from_the_likelier_list() {
    // The nightly updater merges every past solution back into the likelier
    // list, so by today's data the off-list answers are on it and would solve
    // even under the old candidate-list universe. Drop them from the likelier
    // list to reconstruct the state on the day each puzzle ran: that is the
    // situation this change exists to fix, and the only one that regresses if
    // the universe narrows again.
    let mut lexicon = Lexicon::load(FIXTURE_DATA_DIR).unwrap();
    let past = PastSolutionIndex::load(format!("{FIXTURE_DATA_DIR}/past_solutions.json")).unwrap();
    let cases: Vec<_> = known_backtest_cases()
        .into_iter()
        .filter(|case| {
            off_list_backtest_cases()
                .iter()
                .any(|(_, _, answer)| word(answer) == case.answer)
        })
        .collect();
    assert_eq!(cases.len(), off_list_backtest_cases().len());

    for case in &cases {
        lexicon.likelier_solutions.remove(&case.answer);
        assert!(
            lexicon.allowed_guesses.contains(&case.answer),
            "{} must remain an accepted word",
            case.answer
        );
    }

    let games = run_backtest(&lexicon, &past, &cases, 6);
    let unsolved: Vec<String> = games
        .iter()
        .filter(|game| !game.solved)
        .map(|game| game.case.answer.to_string())
        .collect();
    assert!(
        unsolved.is_empty(),
        "answers absent from the likelier list must still be solvable: {unsolved:?}"
    );
}

#[test]
fn accepted_words_outside_the_likelier_list_are_live_candidates() {
    let solver = fixture_solver();
    let off_list: Vec<Word> = solver
        .lexicon()
        .allowed_guesses
        .iter()
        .copied()
        .filter(|word| !solver.lexicon().is_likelier(*word))
        .take(50)
        .collect();
    assert!(!off_list.is_empty());

    let response = solver
        .solve(&solve_request(SolveMode::LikelyAnswer, usize::MAX))
        .unwrap();

    assert_eq!(
        response.remaining_candidates,
        solver.lexicon().allowed_guesses.len()
    );
    // Words the likelier list does not contain must still be rankable answers;
    // that is precisely what the old candidate-list universe made impossible.
    for candidate in off_list {
        let ranked = response
            .likely_answers
            .iter()
            .find(|answer| answer.word == candidate)
            .unwrap_or_else(|| panic!("{candidate} should be rankable"));
        assert!(
            ranked.probability > 0.0,
            "{candidate} should have nonzero probability"
        );
    }
}

#[test]
fn recent_off_list_answers_are_live_candidates() {
    let solver = fixture_solver();
    let response = solver
        .solve(&solve_request(SolveMode::LikelyAnswer, usize::MAX))
        .unwrap();

    // Every accepted word is a candidate now, including answers NYT picked
    // that were never on the likelier-solutions list.
    assert_eq!(
        response.remaining_candidates,
        solver.lexicon().allowed_guesses.len()
    );
    for (_, _, answer) in off_list_backtest_cases() {
        let ranked = response
            .likely_answers
            .iter()
            .find(|candidate| candidate.word == word(answer))
            .unwrap_or_else(|| panic!("{answer} should be rankable"));
        assert!(
            ranked.probability > 0.0,
            "{answer} should have nonzero probability"
        );
    }
}

#[test]
fn plurals_do_not_crowd_out_likelier_answers() {
    let solver = fixture_solver();
    let response = solver
        .solve(&solve_request(SolveMode::LikelyAnswer, 25))
        .unwrap();

    let top: Vec<String> = response
        .likely_answers
        .iter()
        .map(|answer| answer.word.to_string())
        .collect();

    // Off-list words stay live but must not dominate: the list NYT actually
    // draws from should still supply the bulk of the top answers.
    let likelier_count = response
        .likely_answers
        .iter()
        .filter(|answer| solver.lexicon().is_likelier(answer.word))
        .count();
    assert!(
        likelier_count >= 20,
        "expected likelier words to dominate the top 25, got {likelier_count}: {top:?}"
    );
}

#[test]
fn first_turn_cache_matches_uncached_ranking() {
    // A small lexicon exercises the same two code paths as the real word list
    // while keeping the test in milliseconds instead of two full sweeps.
    let words = [
        "slate", "crane", "pride", "brain", "sooty", "shale", "clunk", "geode",
    ];
    let lexicon = Lexicon {
        allowed_guesses: words.iter().map(|w| word(w)).collect(),
        likelier_solutions: ["slate", "crane", "pride"]
            .iter()
            .map(|w| word(w))
            .collect(),
        overrides: EditorialOverrides::default(),
    };
    let past = PastSolutionIndex::from_entries(vec![entry("2026-05-20", 1, "pride")]);
    let uncached = Solver::new(lexicon.clone(), past.clone());
    let cached = Solver::new(lexicon, past).with_first_turn_cache();
    let request = || solve_request(SolveMode::Hybrid, 30);

    let slow = uncached.solve(&request()).unwrap();
    let fast = cached.solve(&request()).unwrap();

    assert_eq!(slow.remaining_candidates, fast.remaining_candidates);
    assert_eq!(
        slow.best_information_guesses.len(),
        fast.best_information_guesses.len()
    );
    assert!(!fast.best_information_guesses.is_empty());
    for (slow_guess, fast_guess) in slow
        .best_information_guesses
        .iter()
        .zip(fast.best_information_guesses.iter())
    {
        assert_eq!(slow_guess.word, fast_guess.word);
        assert_eq!(
            slow_guess.worst_case_remaining,
            fast_guess.worst_case_remaining
        );
        assert_eq!(slow_guess.is_likelier, fast_guess.is_likelier);
        assert!((slow_guess.entropy_bits - fast_guess.entropy_bits).abs() < 1e-9);
        assert!((slow_guess.expected_remaining - fast_guess.expected_remaining).abs() < 1e-9);
        assert!((slow_guess.score - fast_guess.score).abs() < 1e-9);
    }
}
