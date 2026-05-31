use wordle_api::{Lexicon, PastSolutionIndex, known_backtest_cases, run_backtest};

fn main() -> std::io::Result<()> {
    let lexicon = Lexicon::load("wordle-data")?;
    let past = PastSolutionIndex::load("wordle-data/past_solutions.json")?;
    let games = run_backtest(&lexicon, &past, &known_backtest_cases(), 6);

    println!("date,puzzle,answer,solved,guesses_taken,sequence");
    let mut solved = 0usize;
    let mut total_guesses = 0usize;
    let mut worst = 0usize;
    for game in &games {
        let sequence = game
            .guesses
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");
        solved += usize::from(game.solved);
        total_guesses += game.guesses.len();
        worst = worst.max(game.guesses.len());
        println!(
            "{},{},{},{},{},{}",
            game.case.date,
            game.case.puzzle_number,
            game.case.answer,
            game.solved,
            game.guesses.len(),
            sequence
        );
    }
    println!(
        "summary: solved={}/{} average_guesses={:.2} worst_case={}",
        solved,
        games.len(),
        total_guesses as f64 / games.len() as f64,
        worst
    );
    Ok(())
}
