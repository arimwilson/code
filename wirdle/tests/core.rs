use wordle_api::feedback::{LetterStatus::*, pattern_from_statuses};
use wordle_api::filter::GuessInput;
use wordle_api::server::{handle_http_request, parse_solve_request};
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
fn serves_static_ui_as_html() {
    let (status, content_type, response) =
        handle_http_request("GET / HTTP/1.1\r\n\r\n", &fixture_solver());

    assert_eq!(status, "200 OK");
    assert_eq!(content_type, "text/html; charset=utf-8");
    assert!(response.contains("Wordle Solver"));
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
