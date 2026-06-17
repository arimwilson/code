pub mod coach;
pub mod feedback;
pub mod filter;
pub mod lexicon;
pub mod past_solutions;
pub mod rank;
pub mod server;
pub mod solver;
pub mod word;

pub use feedback::{LetterStatus, Pattern, evaluate_feedback};
pub use filter::{filter_candidates, is_candidate_consistent};
pub use lexicon::{EditorialOverrides, Lexicon};
pub use past_solutions::{PastSolutionEntry, PastSolutionIndex};
pub use rank::{InformationGuess, LikelyAnswer};
pub use solver::{
    BacktestCase, BacktestGame, PastSolutionPolicy, SolveMode, SolveRequest, SolveResponse, Solver,
    known_backtest_cases, run_backtest,
};
pub use word::Word;
