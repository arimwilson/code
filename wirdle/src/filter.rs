use crate::feedback::{LetterStatus, evaluate_feedback, pattern_from_statuses};
use crate::word::Word;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuessInput {
    pub word: Word,
    pub statuses: [LetterStatus; 5],
}

impl GuessInput {
    pub fn new(word: Word, statuses: [LetterStatus; 5]) -> Self {
        Self { word, statuses }
    }
}

pub fn is_candidate_consistent(candidate: Word, guesses: &[GuessInput]) -> bool {
    guesses.iter().all(|guess| {
        evaluate_feedback(guess.word, candidate) == pattern_from_statuses(guess.statuses)
    })
}

pub fn filter_candidates(candidates: &[Word], guesses: &[GuessInput]) -> Vec<Word> {
    candidates
        .iter()
        .copied()
        .filter(|candidate| is_candidate_consistent(*candidate, guesses))
        .collect()
}
