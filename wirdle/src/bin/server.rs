use std::env;
use wordle_api::{Solver, server::serve};

fn main() -> std::io::Result<()> {
    let data_dir = env::var("WORDLE_DATA_DIR").unwrap_or_else(|_| "wordle-data".to_string());
    let addr = env::var("WORDLE_ADDR")
        .or_else(|_| env::var("PORT").map(|port| format!("0.0.0.0:{port}")))
        .unwrap_or_else(|_| "127.0.0.1:7878".to_string());
    // Warm the cache before `serve` binds, so requests never see a cold cache.
    let solver = Solver::load(data_dir)?.with_first_turn_cache();
    serve(&addr, solver)
}
