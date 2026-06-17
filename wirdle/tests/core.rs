use wordle_api::coach::{CoachIntent, CoachRequest, coach, status_grid_row};
use wordle_api::feedback::{LetterStatus::*, pattern_from_statuses, statuses_from_pattern};
use wordle_api::filter::GuessInput;
use wordle_api::rank::{evaluate_information_guess, rank_information_guesses, rank_likely_answers};
use wordle_api::server::{handle_http_request, parse_coach_request, parse_solve_request};
use wordle_api::solver::{PastSolutionPolicy, SolveMode, SolveRequest, entry};
use wordle_api::{
    EditorialOverrides, Lexicon, PastSolutionIndex, Solver, Word, evaluate_feedback,
    filter_candidates, known_backtest_cases, run_backtest,
};

fn word(input: &str) -> Word {
    Word::parse(input).unwrap()
}

fn fixture_solver() -> Solver {
    Solver::load("wordle-data").unwrap()
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
        candidate_solutions: vec![word("pride"), word("brain")],
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
                [Present, Present, Present, Present, Present],
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
    let likely = rank_likely_answers(
        &candidates,
        solver.past(),
        &policy,
        &solver.lexicon().overrides,
    );
    let ranked = rank_information_guesses(&[word("slate")], &candidates, &likely, solver.past());
    let evaluated = evaluate_information_guess(word("slate"), &candidates, &likely, solver.past());

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
                [Present, Present, Present, Present, Present],
            )],
            hard_mode: false,
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
        },
    )
    .unwrap_err();

    assert_eq!(err.code, "invalid_request");
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

    let inconsistent = r#"{"intent":"post_game_review","guesses":[{"word":"slate","statuses":["present","present","present","present","present"]}]}"#;
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
fn backtest_solves_last_five_fixture_cases() {
    let lexicon = Lexicon::load("wordle-data").unwrap();
    let past = PastSolutionIndex::load("wordle-data/past_solutions.json").unwrap();
    let games = run_backtest(&lexicon, &past, &known_backtest_cases(), 6);

    assert_eq!(games.len(), 5);
    assert!(games.iter().all(|game| game.solved));
    assert!(games.iter().all(|game| game.guesses.len() <= 6));
}
