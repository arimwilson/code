use std::env;
use wordle_api::{Solver, server::serve};

fn main() -> std::io::Result<()> {
    let data_dir = env::var("WORDLE_DATA_DIR").unwrap_or_else(|_| "wordle-data".to_string());
    let addr = env::var("WORDLE_ADDR").unwrap_or_else(|_| "127.0.0.1:7878".to_string());
    let solver = Solver::load(data_dir)?;
    serve(&addr, solver)
}
