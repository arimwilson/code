# Reconstructing a Wordle game from its shared board

## Status and scope

This document updates the proposed reconstruction design after prototyping it with
Wordle 1,874, whose supplied answer is `GRIPE`. No production implementation is
included in this change. The prototype was an intentionally disposable script in
`/tmp`; it used the checked-in Wirdle lexicons and reproduced Wirdle's existing
feedback and information-ranking formulas.

The feature should estimate a rational, plausible sequence of guesses. It cannot
recover the actual guesses: a shared board contains feedback colors but no guessed
letters, and many allowed words produce every non-green row.

## Prototype result for Wordle 1,874

The board, encoded as absent/present/correct (`0/1/2`), is:

| Turn | Pattern | Compatible allowed guesses | Compatible candidate answers |
| ---: | :---: | ---: | ---: |
| 1 | `00011` | 246 | 63 |
| 2 | `01100` | 387 | 59 |
| 3 | `11002` | 17 | 5 |
| 4 | `02202` | 23 | 11 |
| 5 | `12202` | 5 | 4 |
| 6 | `22222` | 1 | 1 |

Examples demonstrate the ambiguity:

- Turn 1 includes common words such as `SANER`, `ALTER`, `LATER`, `CATER`, and
  `LASER`, as well as many less familiar allowed guesses.
- Turn 2 includes `FIELD`, `FIRST`, `LIGHT`, `NIGHT`, `SPEAK`, and `SPENT`.
- Turn 3's answer-list matches are `PIECE`, `PIQUE`, `PIXIE`, `RIFLE`, and
  `RINSE`.
- Turn 4 includes `BRINE`, `CRIME`, `DRIVE`, `TRIBE`, `URINE`, and `WRITE`.
- Turn 5's answer-list matches are `PRICE`, `PRIDE`, `PRIME`, and `PRIZE`.
- Turn 6 is necessarily `GRIPE`.

A human-plausible reconstruction is therefore:

> `SANER` → `FIELD` → `PIECE` → `CRIME` → `PRICE` → `GRIPE`

This is a useful presentation example, **not** a claim that it is the most
rational path. In the prototype, `SANER` was second among the 246 first-row
matches by the current information formula, leaving 91 candidate solutions.
`FIELD` ranked only 161st among second-row matches for that state and left 15.
`PIECE` then ranked first and reduced the state to `GRIPE` and `TRIPE`.
`CRIME` and `PRICE` did not distinguish those two answers, so a solver optimizing
only information would not prefer that finish. The example exposes exactly why
the product must distinguish “human plausible” from “Wirdle-optimal.”

## Findings from the prototype

### Exact compatibility is easy and valuable

For a known solution, each allowed guess can be placed into a row bucket with:

```text
evaluate_feedback(guess, solution) == shared_pattern
```

This is fast, deterministic, and inherits the existing duplicate-letter rules.
It should be the first filter and a hard invariant for every returned guess.

### Independent row ranking is incorrect

The quality of a turn depends on the candidate state produced by all earlier
guesses. Selecting the most familiar compatible word independently for each row
can create a sequence that looks natural but repeatedly ignores known
information. Reconstruction must search complete paths and recompute the state
after each extension.

### Current information score is not a human-likelihood model

A beam search using only the current entropy-oriented score produced paths headed
by obscure words such as `SOLER`, `DERAT`, `RINCE`, and `BRIZE`. These are legal
and can be informationally strong, but they are poor default predictions of what
a typical rational player typed. Candidate-solution membership alone does not
solve this: ordinary guesses, past solutions, and well-known opener words all
need independent human-usage priors.

### Rational play can conflict with the observed six-turn finish

If an early compatible guess uniquely identifies `GRIPE`, a strictly optimal
player should solve on the following turn, contradicting later non-green rows.
The scorer must therefore model bounded rationality rather than requiring perfect
play. A path may include a familiar but suboptimal guess, while a path that knew
the answer with certainty and ignored it should receive a very large penalty.

### Confidence must express ambiguity, not invented probability

The large compatible sets in turns 1 and 2 make calibrated recovery probabilities
impossible without real guess-frequency data. The UI should use qualitative
labels based on score margins and the number of materially different paths:
`high`, `medium`, `low`, and `many possibilities`.

## Revised product experience

Place **Reconstruct guesses** in Post Game. The form should contain:

1. A textarea for the shared result.
2. A required five-letter solution.
3. A Hard Mode control, defaulting to the marker inferred from the share header.
4. A **Find likely guesses** action.

After parsing, show the color grid before running reconstruction so the user can
catch paste errors. Results should contain:

- one “Most likely rational game” path;
- up to three diverse alternative paths;
- each turn's guess, qualitative confidence, and concise rationale;
- three to five local alternatives per turn;
- an action to substitute an alternative and recompute all downstream turns; and
- a persistent disclosure that the guesses are estimates, not recovered data.

Do not expose the raw internal path score as a probability. For the fixture above,
the UI may show the plausible example path, but it should also explain where its
late turns are strategically weak and offer solver-favored alternatives.

## Revised algorithm

### 1. Parse and validate

Parse the puzzle number, solved score or `X/6`, optional Hard Mode marker, and
five-cell rows. Accept black or white absent squares. Reject malformed row widths,
rows following an all-green row, and a solved score inconsistent with the number
of rows. Validate the supplied solution as an allowed five-letter word.

### 2. Precompute row-compatible guesses

Evaluate every allowed guess once against the supplied solution and bucket it by
feedback pattern. Reject a row with an empty bucket. Force an all-green row to the
solution. This precomputation avoids repeatedly testing the same fixed condition
during search.

### 3. Search paths sequentially

Use beam search over partial guess sequences. Each beam node stores:

- selected guesses;
- the remaining candidate solutions;
- accumulated behavioral score;
- Hard Mode constraints; and
- enough score components to generate rationales.

For each node and turn, intersect the row-compatible bucket with Hard Mode legality
when applicable, score the resulting guesses against that node's candidate state,
apply the observed feedback to create the next state, and retain the best diverse
extensions. Start with a beam width of 64, retain at most 200 locally ranked
extensions per node before pruning, and return five paths. Make these internal
limits configurable for benchmarks rather than API options initially.

### 4. Score bounded-rational human behavior

Use a weighted score with separately inspectable components:

```text
turn_score =
    information_percentile
  + likely_answer_bonus
  + stage_appropriate_probe_or_solve_bonus
  + hard_mode_or_constraint_discipline
  + common_word_prior
  + opener_prior
  - obscurity_penalty
  - unjustified_repeat_penalty
  - known_answer_ignored_penalty
```

Prefer rank percentile to raw entropy so scores are comparable across candidate
set sizes. Change weights by stage: reward exploration early, balance probes and
answers in the middle, and strongly favor a plausible solution late. When one
candidate remains, choosing anything else should be possible only as a heavily
penalized bounded-rationality event.

Human priors should be explicit data, separate from answer priors. For the first
release, check in a versioned common-word/opener table with provenance and allow
editorial overrides. Later, calibrate it from opt-in anonymized Wirdle usage.
Avoid unreviewable letter-shape heuristics as the primary familiarity signal.

### 5. Select diverse outputs and derive confidence

Apply a similarity penalty when selecting final paths so alternatives differ at
meaningful turns. Derive per-turn confidence from the margin between familiar,
rational alternatives and from how many alternatives lie within a score band.
Keep the labels qualitative until evaluation against real completed games supports
calibration.

### 6. Recompute after user edits

When a user chooses a turn alternative, fix the prefix through that choice and
rerun the beam for later turns. Do not splice a new word into the old path because
its candidate state and all later rationality scores have changed.

## Proposed code boundaries

Add `src/reconstruct.rs` with parsing, compatibility bucketing, beam search, and
response types. Expose a small internal turn-analysis abstraction from the current
solver/ranking code so reconstruction can obtain candidates, legal guesses, answer
priors, and full information rankings without copying solver logic.

Add `POST /v1/reconstruct` following the server's existing dependency-free request
parsing and JSON response conventions. Keep the first UI implementation in the
existing static page rather than introducing a framework solely for this feature.

Suggested response concepts are `ReconstructedPath`, `ReconstructedTurn`,
`GuessAlternative`, `Confidence`, and component-level rationale fields. Scores may
be returned for debugging behind a development flag but should not be presented as
probabilities.

## Test and evaluation plan

### Correctness tests

- Parse solved, lost, Hard Mode, black-square, and white-square shares.
- Reject inconsistent scores, invalid row lengths, and rows after a win.
- Assert every returned guess reproduces the exact row against the solution.
- Cover repeated letters in both guess and solution.
- Force the final green row to the supplied solution.
- Assert deterministic ordering under score ties.

### Search tests

- Compare beam output with exhaustive search on a tiny fixture lexicon.
- Verify that changing an early alternative recomputes later candidate states.
- Verify Hard Mode excludes illegal paths.
- Verify path selection returns meaningfully different alternatives.
- Verify a known-answer-ignored path loses to an otherwise comparable solve.

### Behavioral evaluation

Create a versioned, consented set of real completed games, withheld from prior
tuning where possible. Report top-1 and top-5 recovery by turn, separated into
openers, middle game, late game, and Hard Mode. Also report mean solver percentile
of returned guesses and the rate at which obscure words are shown by default.

The Wordle 1,874 fixture should permanently assert the compatibility counts above
and confirm that all proposed paths end in `GRIPE`; it should not assert one
speculative human path as the unique correct reconstruction.

## Delivery sequence

1. **Correctness prototype:** share parser, exact compatibility, candidate-state
   transitions, fixture tests, and a CLI/debug output.
2. **Behavior model:** beam search, rank reuse, explicit human priors, stage-aware
   scoring, Hard Mode, and diversity.
3. **API and UI:** endpoint, Post Game form, parsed preview, rationales,
   alternatives, and downstream recomputation.
4. **Calibration:** evaluate against real games, tune versioned weights, and only
   then reconsider numeric confidence.

This sequence intentionally keeps the next implementation review focused on
correctness before committing the product to subjective ranking weights.
