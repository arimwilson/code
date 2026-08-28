use std::env;
use std::path::PathBuf;
use std::time::Instant;
use wordle_api::solver::FirstTurnStats;
use wordle_api::{Solver, server::serve};

fn main() -> std::io::Result<()> {
    let data_dir = env::var("WORDLE_DATA_DIR").unwrap_or_else(|_| "wordle-data".to_string());
    let addr = env::var("WORDLE_ADDR")
        .or_else(|_| env::var("PORT").map(|port| format!("0.0.0.0:{port}")))
        .unwrap_or_else(|_| "127.0.0.1:7878".to_string());
    let cache_path = env::var("WORDLE_FIRST_TURN_CACHE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(&data_dir).join("first_turn_cache.bin"));

    let solver = Solver::load(&data_dir)?;

    // `--warm-cache`: compute the first-turn statistics and write them to the
    // cache file, then exit. Run at Docker build time so instance cold starts
    // load the cache instead of paying the sweep.
    if env::args().any(|arg| arg == "--warm-cache") {
        let stats = FirstTurnStats::compute(solver.lexicon());
        stats.save(&cache_path, solver.lexicon())?;
        eprintln!(
            "wrote first-turn cache for {} guesses to {}",
            solver.lexicon().allowed_guesses.len(),
            cache_path.display()
        );
        return Ok(());
    }

    // Prefer the build-time cache; fall back to computing when it is missing,
    // torn, or built from different word data. Either way the statistics are
    // ready before `serve` binds, so requests never see a cold cache.
    let load_start = Instant::now();
    let solver = match FirstTurnStats::load(&cache_path, solver.lexicon()) {
        Ok(stats) => {
            eprintln!(
                "loaded first-turn cache from {} in {:?}",
                cache_path.display(),
                load_start.elapsed()
            );
            solver.with_first_turn_stats(stats)
        }
        Err(err) => {
            eprintln!(
                "first-turn cache unusable at {} ({err}); computing at startup",
                cache_path.display()
            );
            solver.with_first_turn_cache()
        }
    };
    serve(&addr, solver)
}
