use crate::feedback::evaluate_feedback;
use crate::lexicon::EditorialOverrides;
use crate::past_solutions::PastSolutionIndex;
use crate::solver::PastSolutionPolicy;
use crate::word::Word;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct LikelyAnswer {
    pub word: Word,
    pub probability: f64,
    pub used_before: bool,
    pub score: f64,
}

#[derive(Clone, Debug)]
pub struct InformationGuess {
    pub word: Word,
    pub entropy_bits: f64,
    pub expected_remaining: f64,
    pub worst_case_remaining: usize,
    pub is_possible_answer: bool,
    pub used_before: bool,
    pub score: f64,
}

pub fn rank_likely_answers(
    candidates: &[Word],
    past: &PastSolutionIndex,
    policy: &PastSolutionPolicy,
    overrides: &EditorialOverrides,
) -> Vec<LikelyAnswer> {
    let positional = positional_letter_priors(candidates);
    let mut weighted: Vec<(Word, f64)> = candidates
        .iter()
        .map(|word| {
            let position_score = word
                .letters()
                .iter()
                .enumerate()
                .map(|(idx, letter)| positional[idx][*letter].max(0.01))
                .product::<f64>();
            let editorial = overrides.word_priors.get(word).copied().unwrap_or(1.0);
            let past_multiplier = past.multiplier(*word, policy);
            (*word, position_score * editorial * past_multiplier)
        })
        .collect();
    let total = weighted
        .iter()
        .map(|(_, weight)| *weight)
        .sum::<f64>()
        .max(f64::EPSILON);
    weighted.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let max = weighted
        .first()
        .map(|(_, weight)| *weight)
        .unwrap_or(1.0)
        .max(f64::EPSILON);
    weighted
        .into_iter()
        .map(|(word, weight)| LikelyAnswer {
            word,
            probability: weight / total,
            used_before: past.was_ever_solution(word),
            score: weight / max,
        })
        .collect()
}

pub fn rank_information_guesses(
    legal_guesses: &[Word],
    candidates: &[Word],
    likely_answers: &[LikelyAnswer],
    past: &PastSolutionIndex,
) -> Vec<InformationGuess> {
    if candidates.is_empty() {
        return Vec::new();
    }
    let mut ranked: Vec<InformationGuess> = legal_guesses
        .iter()
        .copied()
        .map(|guess| evaluate_information_guess(guess, candidates, likely_answers, past))
        .collect();

    ranked.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| b.entropy_bits.total_cmp(&a.entropy_bits))
            .then_with(|| a.worst_case_remaining.cmp(&b.worst_case_remaining))
            .then_with(|| a.word.cmp(&b.word))
    });
    ranked
}

pub fn evaluate_information_guess(
    guess: Word,
    candidates: &[Word],
    likely_answers: &[LikelyAnswer],
    past: &PastSolutionIndex,
) -> InformationGuess {
    if candidates.is_empty() {
        return InformationGuess {
            word: guess,
            entropy_bits: 0.0,
            expected_remaining: 0.0,
            worst_case_remaining: 0,
            is_possible_answer: false,
            used_before: past.was_ever_solution(guess),
            score: 0.0,
        };
    }

    let answer_probability: HashMap<Word, f64> = likely_answers
        .iter()
        .map(|answer| (answer.word, answer.probability))
        .collect();
    let candidate_set: HashSet<Word> = candidates.iter().copied().collect();
    let mut buckets: HashMap<u16, usize> = HashMap::new();
    for candidate in candidates {
        *buckets
            .entry(evaluate_feedback(guess, *candidate))
            .or_insert(0) += 1;
    }

    let candidate_count = candidates.len() as f64;
    let mut entropy = 0.0;
    let mut expected_remaining = 0.0;
    let mut worst_case_remaining = 0usize;
    for bucket_size in buckets.values().copied() {
        let p = bucket_size as f64 / candidate_count;
        entropy -= p * p.log2();
        expected_remaining += p * bucket_size as f64;
        worst_case_remaining = worst_case_remaining.max(bucket_size);
    }

    let is_possible_answer = candidate_set.contains(&guess);
    let answer_prob = answer_probability.get(&guess).copied().unwrap_or(0.0);
    let score = entropy + answer_prob + if is_possible_answer { 0.05 } else { 0.0 }
        - (expected_remaining / candidate_count) * 0.10;

    InformationGuess {
        word: guess,
        entropy_bits: entropy,
        expected_remaining,
        worst_case_remaining,
        is_possible_answer,
        used_before: past.was_ever_solution(guess),
        score,
    }
}

fn positional_letter_priors(candidates: &[Word]) -> [[f64; 26]; 5] {
    let mut counts = [[1.0f64; 26]; 5];
    for word in candidates {
        for (idx, letter) in word.letters().iter().enumerate() {
            counts[idx][*letter] += 1.0;
        }
    }
    let denom = candidates.len() as f64 + 26.0;
    for row in &mut counts {
        for count in row {
            *count /= denom;
        }
    }
    counts
}
