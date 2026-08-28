# Wirdle v2.1: Remove the Candidate Solutions List

**Status: Implemented.** See "Implementation notes" at the end for the two places the built version departs from this design, and for measured results.

## Summary

NYT no longer restricts daily answers to the historical answer slice embedded in its JavaScript assets. Since `candidate_solutions.txt` was created on 2026-07-15, 8 of the ~45 daily answers (`pshaw`, `shill`, `aloha`, `clunk`, `geode`, `aspic`, `runny`, `capon`) were not on the candidate list when they were chosen, so the solver assigned them probability zero and could never suggest them. Every one of those answers was already in `allowed_guesses.txt`, the full NYT accepted-guess list.

This milestone makes the accepted-guess list the solution universe:

- The candidate universe for filtering, remaining-candidate counts, and entropy becomes `allowed_guesses` (14,855 words). No accepted word can ever be "impossible".
- `candidate_solutions.txt` is renamed to `likelier_solutions.txt` and demoted from a hard filter to a soft prior: words on it are weighted 1.0 in likely-answer ranking, all other accepted words 0.2. The 0.2 reflects that off-list answers are now routine (8 of the last ~45) but still much rarer per-word than on-list answers.
- The widened universe makes the first-turn solve ~6.4x slower (measured: 2.24s → 14.41s on this dataset). Mitigation: the server precomputes the expensive, request-independent first-turn statistics at startup, before it begins accepting connections. API requests therefore wait until the cache is complete — the listener simply is not bound until warmup finishes.

## Goals

- Guarantee that every NYT-accepted word is a live solution candidate at all times.
- Keep likely-answer ranking sensible: common, answer-like words must still outrank obscure guess-list words and `-s` plurals.
- Keep first-turn `/v1/solve` latency at or below today's, despite the 6.4x larger universe.
- Preserve the existing HTTP API shapes; only field names and counts change.
- Keep the crate dependency-free and the nightly data pipeline structure intact.

## Non-Goals

- No external word-frequency dataset in this milestone. The likelier/other split plus editorial `word_priors` is the only prior.
- No caching of non-empty board states. Later turns are fast because feedback filtering shrinks the universe quickly.
- No change to the coach hint ladder, share text, or endpoints.
- No speculative NYT answer-slice re-sync; the answer slice stops mattering as a filter.

## Background: verification of the accepted list

The stored `allowed_guesses.txt` (14,855 words) was verified current as of 2026-08-27 two ways: the nightly GitHub Action re-downloads the live NYT JS word list on every run and has produced no changes to the file since its creation on 2026-07-15, and the file is byte-identical to the actively synced `tabatkins/wordle-list` mirror of the NYT accepted list. All 8 off-list answers were present in it before they were chosen.

## Current System

- `scripts/update_wordle_data.py` extracts the full accepted list and the answer slice from NYT JS assets, merges observed past solutions into both, and writes `allowed_guesses.txt` and `candidate_solutions.txt`.
- `src/lexicon.rs` loads both files into `Lexicon { allowed_guesses, candidate_solutions, overrides }` and applies `add_candidate_solutions` / `remove_candidate_solutions` from `editorial_overrides.json`.
- `src/solver.rs` (`Solver::solve`, line 84) filters `lexicon.candidate_solutions` against the observed guesses to get the candidate pool; `rank_likely_answers` and `rank_information_guesses` run over that pool; `health_json` reports both list sizes.
- `src/coach.rs` (`analyze_board`, lines 360/370/390, and the helper at line 1080) uses `lexicon.candidate_solutions` for per-turn candidate pools, information buckets, and trap risk.
- `src/rank.rs` computes positional letter priors over the candidate pool and scores guesses by entropy over the pool.
- `src/server.rs` (`serve`) binds the listener immediately and clones the `Solver` per connection.
- `static/index.html` displays both list counts from `/v1/health` (lines ~1292, ~2186).

The candidate list currently does two jobs at once: it is the filtering universe (which is now wrong — NYT picks outside it) and an implicit "answer-like word" prior (which is still valuable). This design separates the two jobs.

## Design

### 1. Data pipeline

`scripts/update_wordle_data.py`:

- Rename the `candidate_solutions.txt` output to `likelier_solutions.txt`. Content is unchanged: `sorted(answer_slice ∪ past_solution_words)`. The answer slice is still extracted from the NYT JS (it remains a good proxy for "words NYT considers answer-like"); locating `cigar` in the array stays as a sanity check.
- `allowed_guesses.txt` generation is unchanged: `sorted(full_list ∪ answer_slice ∪ past_solution_words)`.
- Rename the editorial override keys `add_candidate_solutions` / `remove_candidate_solutions` to `add_likelier_solutions` / `remove_likelier_solutions`, and update `empty_editorial_overrides()`. Migrate the existing `editorial_overrides.json` by hand in the same commit (the current `aspic` entry is obsolete — it is already merged into the list — so the migrated file starts empty). Words added via `add_likelier_solutions` are still also merged into `allowed_guesses` so an emergency hand-add remains a one-line edit.
- `git mv wordle-data/candidate_solutions.txt wordle-data/likelier_solutions.txt` so history follows the rename.

### 2. Lexicon

`src/lexicon.rs`:

- `Lexicon` becomes:
  - `allowed_guesses: Vec<Word>` — unchanged; now also the solution universe.
  - `likelier_solutions: BTreeSet<Word>` — loaded from `likelier_solutions.txt`; a set because it is only used for membership tests, never iterated as a pool.
  - `overrides: EditorialOverrides` with the renamed fields `add_likelier_solutions` / `remove_likelier_solutions`.
- `Lexicon::load` applies overrides to `likelier_solutions` (and merges adds into `allowed_guesses`), as today.
- Add a helper `likelier_weight(word) -> f64` returning `1.0` if the word is in `likelier_solutions`, else `0.2` (constants `LIKELIER_WEIGHT = 1.0`, `OTHER_ACCEPTED_WEIGHT = 0.2`).

### 3. Solution universe swap

Every candidate-pool read switches to `allowed_guesses`:

- `src/solver.rs:84` — `filter_candidates(&self.lexicon.allowed_guesses, &request.guesses)`.
- `src/coach.rs:360,370,390,1080` — same substitution.
- `tests/core.rs` — fixture `Lexicon` literals and `single_candidate_in_progress_game` updated.

The `no_candidates_remaining` error in `Solver::solve` is kept but should now only trigger on self-contradictory feedback, since the universe equals the legal-guess list.

### 4. Likelier weighting in ranking

`src/rank.rs`:

- `rank_likely_answers` gains access to the lexicon (pass `&Lexicon` instead of only `&EditorialOverrides`). The per-word weight becomes:

  `weight = position_score × likelier_weight × editorial_prior × past_multiplier`

- `positional_letter_priors` is computed over the **likelier subset** of the current candidate pool (falling back to the full pool if the intersection is empty). Computing it over the full universe would let the guess list's thousands of `-s` plurals dominate the priors — measured on current data, the widened pool's top "likely answers" without this change are `sores sanes sones sales seres pares…`, versus `sooty sauce shale saner sleet slant…` today.
- `evaluate_information_guess`: the flat `+0.05` possible-answer bonus currently uses `candidate_set.contains(&guess)`. With the universe equal to the legal-guess list, that is true for every guess on turn one and the bonus degenerates to a constant. Change the bonus to apply when the guess is a **likelier** consistent candidate. `is_possible_answer` itself keeps its meaning ("consistent with all feedback") since it is exposed in API responses and used by the backtest; the backtest's preference for possible answers (solver.rs:209) should likewise prefer likelier possible answers, falling back to any possible answer.

Weight tuning note: 1.0 vs 0.2 is a per-word ratio, decided from the observed 8-of-~45 off-list rate. Because there are ~12,500 non-likelier accepted words vs ~2,360 likelier ones, a 0.2 per-word weight puts roughly half the aggregate first-turn probability mass on off-list words before positional priors are applied — more than the empirical ~18%. The likelier-subset positional priors pull that back down (off-list words skew toward letter patterns the priors penalize), so the net split must be validated by backtest rather than algebra. If off-list words still crowd the top of `likely_answers`, drop `OTHER_ACCEPTED_WEIGHT` toward 0.05 and recheck; the constant should be trivially tunable in one place.

### 5. Startup precomputation of first-turn statistics

The expensive part of a first-turn solve is the entropy loop in `rank_information_guesses`: 14,855 guesses × 14,855 candidates ≈ 220M `evaluate_feedback` calls (measured 14.41s release-mode on current data, vs 2.24s today). The key observation is that for the **empty guess set**, the per-guess bucket statistics — `entropy_bits`, `expected_remaining`, `worst_case_remaining` — depend only on the universe, not on the request. Only the final `score` mixes in request-dependent terms (`answer_prob` from the likely-answer ranking, which varies with `past_solution_policy`), and those are cheap to apply.

Design:

- Add `FirstTurnStats` to `src/solver.rs` (or a new `src/first_turn.rs`): a `Vec` parallel to `allowed_guesses` holding `(entropy_bits, expected_remaining, worst_case_remaining)` per guess, wrapped in `Arc` inside `Solver` so the existing per-connection `Solver::clone` stays cheap.
- `Solver::load` computes it eagerly after loading the lexicon (a `Solver::load_uncached` constructor skips it for tests and tools that never take the fast path). Log the warmup duration to stderr.
- In `Solver::solve`, when `request.guesses.is_empty()` and `hard_mode` imposes no constraints (it never does with zero guesses), skip the entropy loop: build each `InformationGuess` from the cached stats, compute `score` from the request's likely-answer probabilities, then sort and truncate as today. All modes and all `past_solution_policy` values take this fast path. Any request with at least one guess runs the normal loop over the already-filtered (much smaller) pool.
- **Requests wait for the cache.** `src/bin/server.rs` already constructs the `Solver` before calling `serve`, and `serve` binds the `TcpListener` afterwards — so with eager computation in `Solver::load`, no connection can be accepted until the cache is ready. No readiness flag, lock, or 503 path is needed; the ordering is the mechanism. Document in the README that the server takes roughly 15 seconds (current hardware) to begin listening, and that hosted platforms' health-check start periods must exceed the warmup time. If a platform requires the port to open sooner, that is a future change (bind first, hold accepted connections until a `OnceLock` cache fills) — explicitly out of scope now.
- The backtest is unaffected in structure: turn one is a hardcoded `slate` opener, and every solver call it makes has at least one guess. Construct its solver via `load_uncached`. Per-turn cost still rises (14,855 guesses scored against each remaining pool); acceptable for an offline tool.

### 6. API, frontend, docs

- `health_json`: rename the `candidate_solutions` field to `likelier_solutions` (still the list length). `remaining_candidates` in `/v1/solve` responses keeps its name but now starts at 14,855 instead of 2,359.
- `static/index.html`: update the health display (line ~1292) to read "N allowed guess words, M likelier answer words" and the field mapping at line ~2186.
- Coach thresholds: `information_bucket` and `trap_risk` operate on pool sizes/ratios that all grow ~6x at the start of a game. Review both against sample boards and retune the absolute-size cutoffs if any exist; ratio-based logic should survive unchanged.
- README: update the Data section (rename, new semantics of `likelier_solutions.txt`, startup warmup note) and refresh the stale past-solutions row count while there.

## Implementation order

1. **Rename + universe swap** — pipeline output rename, `Lexicon` changes, all `candidate_solutions` use sites, override key migration, test fixture updates. Solver is correct but slow and poorly ranked after this step alone, so land steps 1–3 together.
2. **Likelier weighting** — `rank_likely_answers` weight factor, likelier-subset positional priors, possible-answer bonus change, backtest tiebreak.
3. **First-turn cache** — `FirstTurnStats`, eager load, fast path, `load_uncached`.
4. **Surface updates** — health field, frontend copy, coach threshold review, README.

## Testing

- Unit: `likelier_weight` membership; overrides parsing with renamed keys; fast-path `InformationGuess` output equals slow-path output on a small fixture lexicon (same words, scores within epsilon, identical ordering).
- Ranking regression: with real data, assert each of the 8 recent off-list answers appears in `likely_answers` with nonzero probability on an empty board, and that no `-s` plural of a likelier word outranks the top likelier candidates.
- Latency: assert (in a test or logged benchmark) that a first-turn hybrid solve via the fast path completes well under the current 2.24s baseline.
- Backtest: run `wordle-backtest` before and after; solve rate and average guesses must not regress on the known cases, and add the off-list answers (`pshaw` 2026-07-15 … `capon` 2026-08-26) as new backtest cases — the headline win is that these become solvable.

## Open questions

- Whether 0.2 survives backtest contact (see the weight tuning note); the constant is deliberately isolated so retuning is a one-line change.
- Whether `remaining_candidates` jumping to 14,855 needs frontend copy softening (e.g. showing the likelier-consistent count alongside), since users may read "14,855 possible answers" as a regression. Deferred until the UI is touched.


## Implementation notes

Implemented on `claude/wordle-remove-candidate-list-gw0uay`. Two deviations from the design above, both found while verifying the running server:

### Deviation 1: entropy is prior-weighted, not uniform

The design left `evaluate_information_guess` computing entropy over an unweighted pool and changed only the possible-answer bonus. Verification showed that is not enough. With a uniform distribution over 14,855 words, the recommended guesses became words chosen for splitting the obscure tail: first turn `tares, lares, rales, rates, ranes`, and mid-game `corni, morin, porin` — words a player would not enter.

The fix keeps every accepted word a candidate but measures information against the distribution answers are actually drawn from: each candidate contributes its likelier weight (1.0 or 0.2) to its feedback bucket, and entropy and expected-remaining are computed from those weighted masses. `worst_case_remaining` still uses raw counts, since that is the number of words a player could actually still face. Mid-game recommendations moved from `corni, morin, porin` to `crony, round, corny, groin`.

The weights are static, so this is fully compatible with the startup cache; `FirstTurnStats::compute` applies the identical weighting, and a test asserts the cached and uncached paths agree to 1e-9.

Because `rank_information_guesses` evaluates every guess against the same candidate pool, the per-candidate weight lookup is hoisted out of the per-guess loop (`candidate_weights`). Without that hoist the weighting cost 15.7M redundant `BTreeSet` probes and doubled mid-game latency (1.32s → 2.79s); with it, mid-game is 1.47s.

### Deviation 2: the "impossible board" test fixture changed

Four tests used `slate` with all-present feedback as their inconsistent-board fixture. That board is no longer inconsistent: `taels`, `tales`, and `tesla` are accepted words that satisfy it, and under the new universe they are legitimate candidates. The fixture is now `slate` with the first four letters correct and `e` present, which is impossible for any word in any list (four greens fix every position, so the fifth letter cannot be elsewhere) and so will not break again on a data refresh.

### Measured results

- **Off-list answers are solvable.** All 8 answers that were unsolvable under the old candidate list (`pshaw`, `shill`, `aloha`, `clunk`, `geode`, `aspic`, `runny`, `capon`) now solve in the backtest, averaging 3.25 guesses. They are permanent backtest cases via `off_list_backtest_cases()`.
- **First-turn latency improved over the pre-change baseline**, not just over the widened cost: 2.24s before → ~0.02s served from cache. Startup warmup is 8-9s, and the listener binds only afterwards, so no request ever sees a cold cache.
- **Answer ranking quality held.** Top likely answers are `sooty, sauce, shale, saner, sleet, slant, shone, crier` — effectively identical to the pre-change list. Exactly one off-list word (`sooey`) appears in the top 25, and no plural of a likelier word does.
- **Guess efficiency cost.** On the 5 original backtest cases, average guesses rose from 3.20 to 3.80. This is the expected information-theoretic price of a 6.3x larger universe (log2(14855/2359) ≈ 2.65 bits ≈ half a guess) and is the deliberate trade for never missing an answer.
- 35 tests pass, `cargo fmt --check` is clean.

### Still open

- `OTHER_ACCEPTED_WEIGHT = 0.2` was not retuned; the observed ranking quality did not call for it. It remains a one-line change in `src/lexicon.rs` if off-list words start crowding the top.
- `remaining_candidates` now starts at 14,855 in `/v1/solve` responses. The Word Lists panel explains the new model, but the number itself may still read as a regression to users; deferred as before.
