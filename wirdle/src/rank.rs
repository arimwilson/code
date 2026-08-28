use crate::feedback::evaluate_feedback;
use crate::lexicon::Lexicon;
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
    lexicon: &Lexicon,
) -> Vec<LikelyAnswer> {
    let positional = positional_letter_priors(candidates, lexicon);
    let mut weighted: Vec<(Word, f64)> = candidates
        .iter()
        .map(|word| {
            let position_score = word
                .letters()
                .iter()
                .enumerate()
                .map(|(idx, letter)| positional[idx][*letter].max(0.01))
                .product::<f64>();
            let editorial = lexicon
                .overrides
                .word_priors
                .get(word)
                .copied()
                .unwrap_or(1.0);
            let likelier = lexicon.likelier_weight(*word);
            let past_multiplier = past.multiplier(*word, policy);
            (
                *word,
                position_score * likelier * editorial * past_multiplier,
            )
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
    lexicon: &Lexicon,
) -> Vec<InformationGuess> {
    if candidates.is_empty() {
        return Vec::new();
    }
    // Candidate weights are identical for every guess, so compute them once
    // rather than re-probing the likelier set 14k times per candidate.
    let weights = candidate_weights(candidates, lexicon);
    let mut ranked: Vec<InformationGuess> = legal_guesses
        .iter()
        .copied()
        .map(|guess| {
            evaluate_information_guess_weighted(
                guess,
                candidates,
                &weights,
                likely_answers,
                past,
                lexicon,
            )
        })
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

fn candidate_weights(candidates: &[Word], lexicon: &Lexicon) -> Vec<f64> {
    candidates
        .iter()
        .map(|candidate| lexicon.likelier_weight(*candidate))
        .collect()
}

pub fn evaluate_information_guess(
    guess: Word,
    candidates: &[Word],
    likely_answers: &[LikelyAnswer],
    past: &PastSolutionIndex,
    lexicon: &Lexicon,
) -> InformationGuess {
    let weights = candidate_weights(candidates, lexicon);
    evaluate_information_guess_weighted(guess, candidates, &weights, likely_answers, past, lexicon)
}

fn evaluate_information_guess_weighted(
    guess: Word,
    candidates: &[Word],
    weights: &[f64],
    likely_answers: &[LikelyAnswer],
    past: &PastSolutionIndex,
    lexicon: &Lexicon,
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
    // Buckets carry both a prior-weighted mass (for entropy and expected
    // remaining) and a raw count (for the worst case a player can actually
    // face). Weighting matters because the universe is the whole accepted-guess
    // list: unweighted entropy rewards guesses that split the obscure tail,
    // which is not the distribution answers are drawn from.
    let mut buckets: HashMap<u16, (f64, usize)> = HashMap::new();
    let mut total_weight = 0.0;
    for (candidate, weight) in candidates.iter().zip(weights.iter()) {
        total_weight += weight;
        let bucket = buckets
            .entry(evaluate_feedback(guess, *candidate))
            .or_insert((0.0, 0));
        bucket.0 += weight;
        bucket.1 += 1;
    }

    let total_weight = total_weight.max(f64::EPSILON);
    let mut entropy = 0.0;
    let mut expected_remaining = 0.0;
    let mut worst_case_remaining = 0usize;
    for (bucket_weight, bucket_count) in buckets.values().copied() {
        let p = bucket_weight / total_weight;
        if p > 0.0 {
            entropy -= p * p.log2();
        }
        expected_remaining += p * bucket_count as f64;
        worst_case_remaining = worst_case_remaining.max(bucket_count);
    }
    let candidate_count = candidates.len() as f64;

    let is_possible_answer = candidate_set.contains(&guess);
    let answer_prob = answer_probability.get(&guess).copied().unwrap_or(0.0);
    // The universe is the full accepted-guess list, so `is_possible_answer` is
    // true for nearly every guess early on. Reserve the bonus for words NYT
    // actually draws answers from, or it degenerates into a constant.
    let likelier_possible_answer = is_possible_answer && lexicon.is_likelier(guess);
    let score = entropy + answer_prob + if likelier_possible_answer { 0.05 } else { 0.0 }
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

/// Positional letter priors, computed over the likelier subset of the pool.
///
/// The solution universe is the full accepted-guess list, which is dominated by
/// obscure words and `-s` plurals NYT never picks. Counting those into the
/// priors makes the top likely answers read as noise, so only the likelier
/// words shape the priors. Falls back to the whole pool when no likelier word
/// survives the feedback so far.
fn positional_letter_priors(candidates: &[Word], lexicon: &Lexicon) -> [[f64; 26]; 5] {
    let likelier: Vec<Word> = candidates
        .iter()
        .copied()
        .filter(|word| lexicon.is_likelier(*word))
        .collect();
    let source = if likelier.is_empty() {
        candidates
    } else {
        &likelier
    };

    let mut counts = [[1.0f64; 26]; 5];
    for word in source {
        for (idx, letter) in word.letters().iter().enumerate() {
            counts[idx][*letter] += 1.0;
        }
    }
    let denom = source.len() as f64 + 26.0;
    for row in &mut counts {
        for count in row {
            *count /= denom;
        }
    }
    counts
}
