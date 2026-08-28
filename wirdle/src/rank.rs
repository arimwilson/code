use crate::feedback::evaluate_feedback;
use crate::lexicon::Lexicon;
use crate::past_solutions::PastSolutionIndex;
use crate::solver::PastSolutionPolicy;
use crate::word::Word;
use std::collections::{HashMap, HashSet};

/// Bonus applied to a guess that is both consistent and a word NYT draws
/// answers from.
const LIKELIER_ANSWER_BONUS: f64 = 0.05;
/// Penalty weight on the share of the pool a guess is expected to leave behind.
const EXPECTED_REMAINING_PENALTY: f64 = 0.10;

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
    /// Whether the guess is a word NYT has historically drawn answers from.
    /// With the accepted-guess list as the universe, `is_possible_answer` is
    /// true for nearly every guess early on, so this is the flag consumers
    /// should use to tell answer-shaped guesses from pure probes.
    pub is_likelier: bool,
    pub used_before: bool,
    pub score: f64,
}

/// The request-independent bucket statistics for one guess against one pool.
///
/// This is exactly the part the first-turn cache can precompute, so
/// `FirstTurnStats` stores these directly rather than recomputing the bucket
/// loop.
#[derive(Clone, Copy, Debug, Default)]
pub struct GuessStats {
    pub entropy_bits: f64,
    pub expected_remaining: f64,
    pub worst_case_remaining: usize,
}

/// A candidate pool with its prior weights and the lookups every guess
/// evaluation needs.
///
/// Built once per ranking pass: the weights, the membership set, and the
/// answer-probability map are identical for every guess, so rebuilding them
/// per guess costs tens of millions of redundant inserts on a full-universe
/// pool.
pub struct CandidatePool<'a> {
    words: &'a [Word],
    weights: Vec<f64>,
    total_weight: f64,
    member: HashSet<Word>,
    answer_probability: HashMap<Word, f64>,
}

impl<'a> CandidatePool<'a> {
    pub fn new(words: &'a [Word], lexicon: &Lexicon, likely_answers: &[LikelyAnswer]) -> Self {
        let weights: Vec<f64> = words
            .iter()
            .map(|word| lexicon.likelier_weight(*word))
            .collect();
        let total_weight = weights.iter().sum::<f64>().max(f64::EPSILON);
        Self {
            words,
            weights,
            total_weight,
            member: words.iter().copied().collect(),
            answer_probability: likely_answers
                .iter()
                .map(|answer| (answer.word, answer.probability))
                .collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// Prior-weighted size of the pool.
    ///
    /// Thresholds tuned against the old answer-only universe should compare
    /// against this rather than the raw count: the accepted-guess universe adds
    /// ~12.5k words that each count for a fraction of an answer.
    pub fn effective_size(&self) -> f64 {
        self.total_weight
    }

    pub fn contains(&self, word: Word) -> bool {
        self.member.contains(&word)
    }

    fn answer_probability(&self, word: Word) -> f64 {
        self.answer_probability.get(&word).copied().unwrap_or(0.0)
    }
}

/// Prior-weighted size of a raw candidate slice.
pub fn effective_size(candidates: &[Word], lexicon: &Lexicon) -> f64 {
    candidates
        .iter()
        .map(|word| lexicon.likelier_weight(*word))
        .sum::<f64>()
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
    let pool = CandidatePool::new(candidates, lexicon, likely_answers);
    let mut ranked: Vec<InformationGuess> = legal_guesses
        .iter()
        .copied()
        .map(|guess| {
            let stats = bucket_stats(guess, &pool);
            information_guess(guess, &stats, &pool, past, lexicon)
        })
        .collect();
    sort_information_guesses(&mut ranked);
    ranked
}

/// Bucket a pool by the feedback `guess` would produce, and reduce to entropy,
/// expected remaining, and worst case.
///
/// Buckets carry both a prior-weighted mass (for entropy and expected
/// remaining) and a raw count (for the worst case a player can actually face).
/// Weighting matters because the universe is the whole accepted-guess list:
/// unweighted entropy rewards guesses that split the obscure tail, which is not
/// the distribution answers are drawn from.
pub fn bucket_stats(guess: Word, pool: &CandidatePool<'_>) -> GuessStats {
    if pool.is_empty() {
        return GuessStats::default();
    }

    let mut buckets: HashMap<u16, (f64, usize)> = HashMap::new();
    for (candidate, weight) in pool.words.iter().zip(pool.weights.iter()) {
        let bucket = buckets
            .entry(evaluate_feedback(guess, *candidate))
            .or_insert((0.0, 0));
        bucket.0 += weight;
        bucket.1 += 1;
    }

    let mut stats = GuessStats::default();
    for (bucket_weight, bucket_count) in buckets.values().copied() {
        let p = bucket_weight / pool.total_weight;
        if p > 0.0 {
            stats.entropy_bits -= p * p.log2();
        }
        stats.expected_remaining += p * bucket_count as f64;
        stats.worst_case_remaining = stats.worst_case_remaining.max(bucket_count);
    }
    stats
}

/// Score a guess from its bucket statistics.
///
/// The only request-dependent input is the answer probability, which is why the
/// first-turn cache can store `GuessStats` and apply this at request time.
pub fn score_guess(
    stats: &GuessStats,
    answer_prob: f64,
    likelier_possible_answer: bool,
    pool_len: usize,
) -> f64 {
    let bonus = if likelier_possible_answer {
        LIKELIER_ANSWER_BONUS
    } else {
        0.0
    };
    stats.entropy_bits + answer_prob + bonus
        - (stats.expected_remaining / pool_len.max(1) as f64) * EXPECTED_REMAINING_PENALTY
}

/// Assemble a scored `InformationGuess` from precomputed bucket statistics.
pub fn information_guess(
    guess: Word,
    stats: &GuessStats,
    pool: &CandidatePool<'_>,
    past: &PastSolutionIndex,
    lexicon: &Lexicon,
) -> InformationGuess {
    let is_possible_answer = pool.contains(guess);
    let is_likelier = lexicon.is_likelier(guess);
    InformationGuess {
        word: guess,
        entropy_bits: stats.entropy_bits,
        expected_remaining: stats.expected_remaining,
        worst_case_remaining: stats.worst_case_remaining,
        is_possible_answer,
        is_likelier,
        used_before: past.was_ever_solution(guess),
        score: score_guess(
            stats,
            pool.answer_probability(guess),
            is_possible_answer && is_likelier,
            pool.len(),
        ),
    }
}

pub fn sort_information_guesses(ranked: &mut [InformationGuess]) {
    ranked.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| b.entropy_bits.total_cmp(&a.entropy_bits))
            .then_with(|| a.worst_case_remaining.cmp(&b.worst_case_remaining))
            .then_with(|| a.word.cmp(&b.word))
    });
}

pub fn evaluate_information_guess(
    guess: Word,
    candidates: &[Word],
    likely_answers: &[LikelyAnswer],
    past: &PastSolutionIndex,
    lexicon: &Lexicon,
) -> InformationGuess {
    let pool = CandidatePool::new(candidates, lexicon, likely_answers);
    if pool.is_empty() {
        return InformationGuess {
            word: guess,
            entropy_bits: 0.0,
            expected_remaining: 0.0,
            worst_case_remaining: 0,
            is_possible_answer: false,
            is_likelier: lexicon.is_likelier(guess),
            used_before: past.was_ever_solution(guess),
            score: 0.0,
        };
    }
    let stats = bucket_stats(guess, &pool);
    information_guess(guess, &stats, &pool, past, lexicon)
}

/// Positional letter priors, computed over the likelier subset of the pool.
///
/// This is deliberately a hard subset rather than a `likelier_weight` blend,
/// unlike every other use of the prior. The priors describe the *shape* of an
/// answer word, and the accepted-guess universe holds ~5x more off-list words
/// than likelier ones — at weight 0.2 they still out-mass the likelier words
/// (12,496 x 0.2 > 2,359) and the top answers collapse back into `-s` plurals
/// (`sores`, `sanes`, `sones`). Measured: blending puts 11 plurals in the top
/// 25; the subset puts none. Falls back to the whole pool when no likelier word
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
