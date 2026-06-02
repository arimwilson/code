# Rust Wordle Solver API Design Plan

_Last updated: 2026-05-29_

## 1. Goal

Build a Rust HTTP API that accepts the current Wordle board state — every guess made so far plus the status of each letter — and returns:

1. **Likely answers**: words most likely to be the hidden solution.
2. **Information-maximizing guesses**: legal guesses that best split the remaining candidate space, even when they are not likely answers.

The API should ingest a dated list of words already used as Wordle solutions and **down-weight those words heavily**, rather than excluding them. This matters because Wordle now repeats some historical answers.

## 2. Important Product Decision: Down-weight, Do Not Eliminate

Previously, many solvers eliminated any word that had already appeared as a Wordle answer. That is no longer safe. The design should treat historical answers as less likely but still possible.

Recommended default policy:

```json
{
  "enabled": true,
  "weight_multiplier": 0.05,
  "recent_repeat_multiplier": 0.01,
  "recent_days": 90
}
```

Interpretation:

- If a word has never been used as a solution, multiplier = `1.0`.
- If a word was used before, multiplier = `0.05`.
- If a word was used very recently, multiplier = `0.01`.
- The multiplier should be configurable per request and globally through service config.

## 3. Data Inputs

The service should load four data files at startup.

```text
wordle-data/
  allowed_guesses.txt
  candidate_solutions.txt
  past_solutions.json
  editorial_overrides.json
```

### 3.1 `allowed_guesses.txt`

All five-letter words accepted as guesses. This list is larger than the possible-answer list.

Example source for development fixtures:

- `https://raw.githubusercontent.com/Roy-Orbison/wordle-guesses-answers/main/guesses.txt`

### 3.2 `candidate_solutions.txt`

The current candidate answer list. This must be updateable because the NYT editor can add words that were not in the older static answer list.

Example source for development fixtures:

- `https://raw.githubusercontent.com/Roy-Orbison/wordle-guesses-answers/main/answers.txt`

Important finding from the backtest below: `DIVOT`, the May 28, 2026 answer, was accepted as a guess in the fixture list but was not present in the static `answers.txt` fixture I used. Production should therefore treat the answer list as a refreshable input, not an immutable historical list.

### 3.3 `past_solutions.json`

Dated historical answers used for the prior penalty.

Recommended format:

```json
[
  {
    "date": "2026-05-29",
    "puzzle_number": 1805,
    "solution": "clang",
    "source": "techradar/parade/wordfinder/etc",
    "is_repeat": false
  }
]
```

Rules:

- Store lowercase ASCII words.
- Store date and puzzle number.
- Allow repeats.
- Do not assume one solution per date forever; Wordle has had exceptional days with different answers for different clients.
- Cross-check against at least two public archives when possible.
- For backtesting, use an **as-of snapshot**: when simulating puzzle `D`, the past-solutions DB must include only solutions strictly before `D`.

### 3.4 `editorial_overrides.json`

A lightweight manual override file for NYT/editorial behavior. `candidate_solutions.txt`
is the canonical generated answer list and should already include observed historical
solutions; the updater should not auto-populate runtime candidate additions.

```json
{
  "add_candidate_solutions": [],
  "remove_candidate_solutions": [],
  "word_priors": {
    "snafu": 0.8,
    "guana": 0.8
  }
}
```

## 4. API Surface

### 4.1 `POST /v1/solve`

Request:

```json
{
  "guesses": [
    {
      "word": "slate",
      "statuses": ["absent", "present", "absent", "correct", "absent"]
    },
    {
      "word": "crony",
      "statuses": ["absent", "absent", "correct", "absent", "absent"]
    }
  ],
  "mode": "hybrid",
  "hard_mode": false,
  "limit": 20,
  "past_solution_policy": {
    "enabled": true,
    "weight_multiplier": 0.05,
    "recent_repeat_multiplier": 0.01,
    "recent_days": 90
  }
}
```

Response:

```json
{
  "remaining_candidates": 17,
  "likely_answers": [
    {
      "word": "pride",
      "probability": 0.183,
      "used_before": false,
      "score": 0.812
    }
  ],
  "best_information_guesses": [
    {
      "word": "grime",
      "entropy_bits": 3.94,
      "expected_remaining": 2.1,
      "worst_case_remaining": 5,
      "is_possible_answer": false,
      "used_before": false,
      "score": 0.903
    }
  ]
}
```

### 4.2 `GET /v1/health`

Response:

```json
{
  "ok": true,
  "candidate_solutions": 2310,
  "allowed_guesses": 14855,
  "past_solutions": 1805,
  "data_updated_at": "2026-05-29T12:00:00Z"
}
```

### 4.3 `GET /v1/metadata`

Expose non-sensitive model metadata useful for debugging:

```json
{
  "data_version": "2026-05-29",
  "supports_repeated_answers": true,
  "default_past_solution_policy": {
    "enabled": true,
    "weight_multiplier": 0.05,
    "recent_repeat_multiplier": 0.01,
    "recent_days": 90
  }
}
```

## 5. Rust Domain Model

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LetterStatus {
    Correct,
    Present,
    Absent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuessInput {
    pub word: String,
    pub statuses: [LetterStatus; 5],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolveMode {
    LikelyAnswer,
    MaxInformation,
    Minimax,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastSolutionPolicy {
    pub enabled: bool,
    pub weight_multiplier: f64,
    pub recent_repeat_multiplier: f64,
    pub recent_days: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolveRequest {
    pub guesses: Vec<GuessInput>,
    pub mode: SolveMode,
    pub hard_mode: bool,
    pub limit: usize,
    pub past_solution_policy: PastSolutionPolicy,
}
```

Use a compact internal word representation:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Word([u8; 5]);

// Feedback pattern packed in base-3.
// 0 = absent, 1 = present, 2 = correct.
pub type Pattern = u16;
```

## 6. Correct Feedback Evaluation

Avoid hand-written constraint logic for green/yellow/gray rules. Instead, evaluate whether each candidate solution would produce exactly the same feedback pattern as the user's observed board.

Algorithm:

1. Mark exact matches as green.
2. Count remaining unmatched letters in the answer.
3. Mark yellows only while unmatched answer counts remain.
4. Mark everything else gray.

```rust
pub fn evaluate_feedback(guess: Word, answer: Word) -> Pattern {
    // 1. Mark greens.
    // 2. Count unmatched answer letters.
    // 3. Mark yellows if count remains.
    // 4. Pack into base-3 Pattern.
    todo!()
}

pub fn is_candidate_consistent(candidate: Word, guesses: &[GuessInput]) -> bool {
    guesses.iter().all(|guess| {
        evaluate_feedback(Word::from(&guess.word), candidate) == Pattern::from(guess.statuses)
    })
}
```

This approach handles duplicates correctly. For example, if the guess is `eerie` and the answer has only one `e`, the evaluator must not mark every `e` yellow.

## 7. Candidate Filtering

```rust
pub fn filter_candidates(
    candidates: &[Word],
    guesses: &[GuessInput],
) -> Vec<Word> {
    candidates
        .iter()
        .copied()
        .filter(|candidate| is_candidate_consistent(*candidate, guesses))
        .collect()
}
```

If no candidates remain, return a structured error:

```json
{
  "error": "no_candidates_remaining",
  "message": "No candidate solution matches the provided guesses and statuses. Check duplicate-letter feedback."
}
```

## 8. Likely-answer Ranking

Each candidate gets a posterior-ish weight:

```text
weight(word) =
    base_word_prior(word)
  × positional_letter_prior(word)
  × editorial_prior(word)
  × past_solution_multiplier(word)
```

Past solution multiplier:

```rust
pub fn past_solution_multiplier(
    word: Word,
    past: &PastSolutionIndex,
    policy: &PastSolutionPolicy,
) -> f64 {
    if !policy.enabled {
        return 1.0;
    }

    if past.was_recent_solution(word, policy.recent_days) {
        policy.recent_repeat_multiplier
    } else if past.was_ever_solution(word) {
        policy.weight_multiplier
    } else {
        1.0
    }
}
```

The service should return the raw `used_before` flag alongside the score so the UI can explain why a plausible word is ranked lower.

## 9. Information Ranking

For each legal guess, partition the remaining candidate solutions by the feedback pattern that guess would produce.

There are at most:

```text
3^5 = 243
```

possible feedback patterns.

Entropy score:

```text
entropy(guess) = -Σ p(pattern) × log2(p(pattern))
```

Expected remaining candidates:

```text
expected_remaining(guess) = Σ p(pattern) × bucket_size(pattern)
```

Worst-case remaining candidates:

```text
worst_case_remaining(guess) = max(bucket_size(pattern))
```

Return all three because they answer different questions:

- `entropy_bits`: best information gain on average.
- `expected_remaining`: easiest for product/UI explanation.
- `worst_case_remaining`: useful for minimax-style play.

## 10. Scoring Modes

### 10.1 `likely_answer`

Only ranks remaining candidate answers by answer probability.

Use when the pool is small and the user wants to solve now.

### 10.2 `max_information`

Ranks all legal guesses by entropy.

Use early in the game or when the candidate space is large.

### 10.3 `minimax`

Ranks guesses by minimizing the worst-case bucket size.

Useful when many candidates share a family like:

```text
?ight
?ound
?atch
```

### 10.4 `hybrid`

Good default.

```text
hybrid_score =
    0.60 × normalized_entropy
  + 0.25 × normalized_answer_probability
  - 0.10 × normalized_expected_remaining
  + 0.05 × is_candidate_bonus
```

Tune these weights by simulation.

## 11. Project Structure

```text
wordle-api/
  Cargo.toml
  crates/
    wordle-core/
      src/
        word.rs
        feedback.rs
        filter.rs
        rank.rs
        entropy.rs
        lexicon.rs
        past_solutions.rs
        simulation.rs
    wordle-server/
      src/
        main.rs
        routes.rs
        models.rs
        errors.rs
    wordle-data/
      allowed_guesses.txt
      candidate_solutions.txt
      past_solutions.json
      editorial_overrides.json
```

Recommended crates:

```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
rayon = "1"
dashmap = "6"
utoipa = "4"
utoipa-swagger-ui = "6"
```

## 12. Precomputation

The expensive operation is:

```text
for every legal guess:
  for every candidate solution:
    compute feedback pattern
```

Precompute:

```text
pattern_cache[guess_id][candidate_id] -> Pattern
```

Approximate memory:

```text
~15,000 guesses × ~2,500 candidates × 2 bytes ≈ 75 MB
```

This is acceptable for a small API server and makes per-request ranking fast.

## 13. Data Updater CLI

Add a CLI binary:

```bash
wordle-data update-past-solutions
wordle-data validate
wordle-data diff
wordle-data backtest --from 2026-05-25 --to 2026-05-29
```

Validation rules:

- every word is five letters
- lowercase ASCII only
- dates are valid
- puzzle numbers are monotonic except known exceptions
- repeated answers are allowed
- source disagreements become warnings, not silent overwrites
- current-day answer is never included in the as-of past-solutions DB when backtesting that day

## 14. Backtesting Method

### 14.1 Purpose

Estimate how many guesses this solver strategy would have taken for the last five known Wordle puzzles, without leaking the true answers through the past-solutions DB.

### 14.2 Inputs

Verified last-five answers used for this test:

| Puzzle | Date | Answer |
|---:|---|---|
| 1801 | 2026-05-25 | VISIT |
| 1802 | 2026-05-26 | COUCH |
| 1803 | 2026-05-27 | STUFF |
| 1804 | 2026-05-28 | DIVOT |
| 1805 | 2026-05-29 | CLANG |

### 14.3 Leakage prevention

For each simulated puzzle date:

- The target answer remained in the candidate solution set; otherwise solving would be impossible.
- The target answer was excluded from the `past_solutions` penalty DB for that simulated day.
- Later real answers were also excluded from the `past_solutions` DB.
- The solver did not get the chronological solution list for the test window as a ranking feature.

### 14.4 Backtest strategy used

This is a lightweight local simulation of the proposed algorithm, not a final optimized solver.

- Opening word: `SLATE`.
- Candidate set: public answer fixture plus any observed daily answers missing from the static fixture.
- Legal guesses: public valid-guess fixture.
- Feedback: exact Wordle duplicate-letter evaluation.
- After each guess: filter candidates by exact feedback pattern.
- If remaining candidates > 2: choose a high-entropy candidate/guess.
- If remaining candidates <= 2: choose the highest-prior remaining candidate.
- Historical past-solution penalty: disabled in the numeric quick test to avoid introducing leakage from an incomplete dated archive. Production backtests should enable it with an as-of historical archive.

### 14.5 Backtest results

| Date | Answer | Guesses taken | Guess sequence | Notes |
|---|---:|---:|---|---|
| 2026-05-25 | VISIT | 3 | `SLATE` → `TROIS` → `VISIT` | `SLATE` left 24 candidates. |
| 2026-05-26 | COUCH | 3 | `SLATE` → `CRONY` → `COUCH` | `SLATE` was all gray and left 221 candidates. |
| 2026-05-27 | STUFF | 4 | `SLATE` → `UNPOT` → `SAUCY` → `STUFF` | Repeated `F` made this harder. |
| 2026-05-28 | DIVOT | 3 | `SLATE` → `ROUND` → `DIVOT` | `DIVOT` was not in the static answer fixture, so it had to be added from observed solution data. |
| 2026-05-29 | CLANG | 3 | `SLATE` → `CRINK` → `CLANG` | `SLATE` left 20 candidates. |

Summary:

```text
Solved: 5 / 5
Average guesses: 3.2
Worst case in this window: 4 guesses
Best case in this window: 3 guesses
```

### 14.6 Caveats

- The quick test is not a fully fair historical replay unless it uses an actual dated `past_solutions.json` snapshot as of each puzzle date.
- The quick test did not use NYT WordleBot's private dictionary or its private scoring model.
- The static public answer fixture was stale for `DIVOT`, which reinforces the need for the API to ingest observed solution data continuously.
- The production backtester should compare at least these modes: `likely_answer`, `max_information`, `minimax`, and `hybrid`.

## 15. Tests to Prioritize

```text
- duplicate letters in guess
- duplicate letters in answer
- all-gray repeated letters
- yellow cannot reuse consumed answer letters
- hard-mode validation
- past solution penalty does not eliminate valid candidates
- repeated answers remain possible
- current-day answer excluded from past-solution DB during backtests
- entropy ranking handles one remaining candidate
- no-candidates error is clear and actionable
```

## 16. Implementation Order

1. Implement `Word` and parsing/validation.
2. Implement exact feedback pattern evaluation.
3. Implement candidate filtering.
4. Load `allowed_guesses.txt` and `candidate_solutions.txt`.
5. Load `past_solutions.json` and apply configurable prior penalty.
6. Implement likely-answer ranking.
7. Implement entropy and expected-remaining ranking.
8. Add `hybrid` scoring.
9. Add `POST /v1/solve` with structured errors.
10. Add backtest runner with as-of snapshots.
11. Add updater CLI for historical answers.
12. Add OpenAPI docs.
13. Add benchmarks and precomputed pattern cache.

## 17. Source Notes

Public sources used while drafting this plan:

- TechRadar past Wordle answers archive: `https://www.techradar.com/news/past-wordle-answers`
- TechRadar best starting words / WordleBot starter discussion: `https://www.techradar.com/how-to/wordle-best-starting-word`
- Parade May 29, 2026 Wordle answer page: `https://parade.com/living/wordle-today-hint-answer-1805-day-may-29-2026`
- Public fixture lists:
  - `https://raw.githubusercontent.com/Roy-Orbison/wordle-guesses-answers/main/answers.txt`
  - `https://raw.githubusercontent.com/Roy-Orbison/wordle-guesses-answers/main/guesses.txt`
