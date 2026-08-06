# Shared Board Reconstruction — Design and Algorithm Findings

## Summary

Given a shared Wordle grid (the emoji block) and the puzzle's answer, reconstruct the
most likely guess words the player actually typed.

The naive framing — "filter the lexicon by each row's pattern, then pick the highest-scoring
word per row" — fails on real boards. Prototyping against Wordle 1,874 (`GRIPE`) showed the
row-local view is both under-determined early (246 and 387 words fit rows 1 and 2) and
*degenerate* late: by turn 4 the solution-list candidate set has collapsed to a single word,
at which point every legal guess scores identically and the model has no opinion at all.

The working algorithm models the **player's** knowledge state rather than the solver's, and
searches over whole sequences so that a constraint violation on turn 4 can retroactively rule
out a word on turn 3. That coupling turns out to be the single most valuable signal available.

A second board with a *known* ground-truth sequence (below) then falsified the word prior:
using "is this word a Wordle answer?" as a proxy for "would a player type this word?" demoted
the single most informative guess on the board from **#1 to #24**. The information model was
sound; the prior was wrong, and in a systematic direction. See
[Validation against ground truth](#validation-against-ground-truth-and-where-the-model-failed).

## Problem statement

Given answer `A` and observed patterns `P₁..P_T`, find `g₁..g_T` maximising plausibility
subject to `evaluate_feedback(gᵢ, A) == Pᵢ` for every turn.

## Algorithm

### Step 1 — bucket once per answer

One pass over `allowed_guesses` computing `evaluate_feedback(w, A)` yields every row's legal
set simultaneously: `Sᵢ = bucket[Pᵢ]`. 14,855 evaluations total, negligible. An empty `Sᵢ`
means the grid is impossible for that answer — return 422 naming the offending row.

Measured on the `GRIPE` board:

| turn | pattern | \|Sᵢ\| | of which in solution list |
|---|---|---|---|
| 1 | ⬛⬛⬛🟨🟨 | 246 | 63 |
| 2 | ⬛🟨🟨⬛⬛ | 387 | 59 |
| 3 | 🟨🟨⬛⬛🟩 | 17 | 5 |
| 4 | ⬛🟩🟩⬛🟩 | 23 | 11 |
| 5 | 🟨🟩🟩⬛🟩 | 5 | 4 |
| 6 | 🟩🟩🟩🟩🟩 | 1 | 1 |

All real branching lives in rows 1–2. An all-green final row forces `g_T = A`.

### Step 2 — model the player's belief set, not the solver's

The first prototype scored guesses with `rank::evaluate_information_guess` against the
solution-list candidate set `K` (2,354 words). Measured collapse along a plausible line:

```
before turn 1: |K|=2354      before turn 4: |K|=1
before turn 2: |K|= 100      before turn 5: |K|=1
before turn 3: |K|=  22      before turn 6: |K|=1
```

By turn 4 `K = {gripe}`, so entropy is 0 for every guess, `expected_remaining/|K|` is 1 for
every guess, and **all 23 legal turn-4 words score exactly `-0.1000`**. The information model
is blind precisely where the human was still making choices.

The cause is a modelling error, not a bug: the player is not enumerating a 2,354-word list.
They are thinking of words they know. So the belief set is drawn from `allowed_guesses`
(14,855) weighted by familiarity:

```
B₁ = allowed_guesses,  familiarity(w) = <see below>
Bᵢ₊₁ = { w ∈ Bᵢ : feedback(w, gᵢ) == Pᵢ }
```

and the information term becomes a familiarity-weighted entropy over `Bᵢ`. Entropy stays
non-zero into the endgame and the model regains discrimination.

> **Correction.** The prototype used `familiarity(w) = 1.0 if w ∈ candidate_solutions else 0.15`.
> That is wrong and was the largest single source of error — see
> [Validation against ground truth](#validation-against-ground-truth-and-where-the-model-failed).
> Use a frequency-derived familiarity instead.

### Step 3 — constraint discipline is the dominant late signal

Track what the player can literally see: green positions, letters known present, letters
known absent. Penalise, in the per-turn score:

| violation | penalty |
|---|---|
| abandoning a known green position | 3.0 |
| dropping a letter known to be present | 2.5 |
| re-testing a letter known absent | 1.2 |

This is worth more than any tuning of the entropy term, because it **propagates backwards
through the sequence**. Concretely, on the `GRIPE` board the turn-3 set splits into words
starting `p…` (`piece`, `pique`, `pixie`) and words starting `r…` (`rifle`, `rinse`, `rince`):

```
if turn3 = piece -> letters known present {i,p} -> hard-mode-legal turn-4 words:  0
if turn3 = rinse -> letters known present {i,r} -> hard-mode-legal turn-4 words: 23
```

*No* word in `S₄` contains a `p`. So if turn 3 had been `piece`, turn 4 must have thrown away
a known-present letter. A player assumed to be rational would not, so the turn-4 penalty
retroactively demotes `piece` on turn 3. The pure-entropy prototype ranked `piece` first;
with the penalty term it drops out of the top six entirely. **This is the reason to search
over sequences rather than rows.**

(The same test doubles as a hard-mode detector: if every reconstruction requires a violation,
the board was not played in hard mode.)

### Step 4 — beam search, stratified

State is `(guesses so far, belief set Bᵢ, visible constraints, log-likelihood)`.
Per turn, per state:

```
logPr(g) ∝ β · [ H(g; Bᵢ) + 0.05·1(g ∈ Bᵢ) ] − penalty(g; constraints) + log prior(g)
```

normalised by log-sum-exp over the shortlist. `β` is the rationality temperature (default 1.5;
β→∞ is "pure Wirdle bot"). `prior` is familiarity, times an opener-popularity table on turn 1.
Shortlist `Sᵢ` to the top ~350 by static prior, unioned with all of `Sᵢ ∩ Bᵢ`.

**Stratify the prune.** The first prototype reported turn-1 confidence of 1.00 — an artefact:
the beam kept 24 states that all descended from one opening word, so marginalising over the
beam produced false certainty. Capping slots per distinct opening word (6 of 48) fixes it and
yields the honest turn-1 spread below. Any beam-marginal confidence figure is untrustworthy
without this.

### Step 5 — marginalise

Per-turn word probability is the sum of sequence probabilities over the final beam, not the
row-local softmax. This accounts for downstream consistency, which is the whole point.

## Result on the first board (Wordle 1,874, GRIPE, 6/6 — no ground truth)

```
most likely sequence:  saner  beret  rifle  crime  pride  gripe

turn 1 (|S|=246): saner 0.25   later 0.17   stair 0.12   loser 0.12   laser 0.09
turn 2 (|S|=387): beret 0.24   verso 0.16   birch 0.12   terra 0.12   berry 0.10
turn 3 (|S|= 17): rifle 0.43   rince 0.26   rinse 0.19   rione 0.09
turn 4 (|S|= 23): crime 0.37   bride 0.36   drive 0.21   tribe 0.03   urine 0.03
turn 5 (|S|=  5): prize 0.42   prime 0.25   pride 0.25   price 0.07
turn 6 (|S|=  1): gripe 1.00
```

Reading the model's behaviour, which is a good sanity check on its priors:

- Turn 2 prefers `beret` over `verso` because after `saner` the player knows `e` and `r` are
  present and `s` is absent; `beret` uses both known letters, `verso` re-tests the dead `s`.
- Turn 3 collapses onto `r…` words for the backward-propagation reason above.
- Turn 4 suppresses `urine` and `arise` — both re-test letters already known absent.
- Turns 1–2 stay genuinely uncertain (max 0.25). That is the correct answer, not a failure;
  246 words fit row 1 and nothing in the board distinguishes them. The UI must say so.

**Honest confidence:** turns 3–6 are well-determined, turns 1–2 are a coin flip among a
handful. Reported to a user, this board should read as "turn 3 was almost certainly `rifle`,
turn 5 was a `pri_e` word — but the opening two guesses can't be pinned down."

## Validation against ground truth, and where the model failed

A second shared board for the same puzzle, this time with the real sequence known:

```
Wordle 1,874 4/6        truth:      TEARS  PORED  CREPE  GRIPE
⬛🟨⬛🟨⬛                model said:  media  inert  crepe  gripe
🟨⬛🟨🟨⬛
⬛🟩⬛🟩🟩                row sets:     442     56      7      1
🟩🟩🟩🟩🟩
```

One of three reconstructible turns correct. The failure is diagnosable and systematic.

### Root cause: answer-list membership is the wrong prior for a guess

**`TEARS` and `PORED` are not in `candidate_solutions`.** Wordle's answer list editorially
excludes ‑S plurals and most ‑ED past tenses. The binary familiarity prior therefore scored
them `0.15` — the same bucket as `frape`, `tiars`, `porey`.

| word | entropy rank in its row | rank after the prior | reason demoted |
|---|---|---|---|
| `tears` | **#1 of 442** (6.01 bits) | #24 | ‑S plural, not an answer word |
| `pored` | #2 of 56 (4.34 bits) | #4 | ‑ED past tense, not an answer word |
| `crepe` | #1 of 7 | #1 ✓ | in the answer list |

`tears` was the most informative guess available out of all 14,855 allowed words, and the
prior ranked it 24th. The damage is broader than one word: the entropy-only top of row 1 was
*entirely* ‑S words — `tears, lears, nears, dears, hears` — so the ×0.15 factor demoted the
whole correct region of the search space and surfaced `media`/`learn`/`teary` instead.

The two distributions are not merely different, they are **anti-correlated for probes**: ‑S
and ‑ED endings are strong information plays *precisely because* NYT never uses them as
answers, so players spend them freely. `candidate_solutions` membership belongs only in the
"this guess might itself be the answer" term, never in the guess prior.

A second, self-inflicted error compounded it: the turn-1 opener table was hand-invented, gave
`media` a ×2.5 boost on no evidence, and omitted `tears`, a well-known opener.

### The fix, measured

Replacing the binary prior with a real frequency corpus (`wordfreq`, Zipf scale):

```
familiarity(w) = max(0.01, 10^(γ · (zipf(w) − 3.0)) · (1.6 if w ∈ candidate_solutions else 1))
γ = 0.55
```

| word | before | after |
|---|---|---|
| `tears` | #24 of 442 | **#6** |
| `pored` | #4 of 56 | **#5** |
| `crepe` | #1, p = 0.72 | **#1, p = 0.93** |

A clear improvement, but it does **not** recover the sequence, and the reasons are worth
recording rather than tuning away:

- Turn 1: `years` (zipf 5.96) and `heart` (5.31) are more common than `tears` (4.47) and
  nearly as informative. Frequency alone cannot separate them.
- Turn 2: `roger` takes p = 0.60 as a pure corpus artefact — frequent as a name and a radio
  idiom, not as a word players type. Proper nouns and interjections inflate Zipf. Cap the
  frequency term's dynamic range so it cannot override a ~0.5-bit entropy gap.

### Dead end: system dictionaries are not a commonality oracle

`/usr/share/dict/words` (web2, 235k entries) was tried as a dependency-free substitute and is
unusable in **both** directions: it is a headword list, so it lacks `tears` and `roped`
entirely, while including `lear` and `tiar` as headwords. Stem-based inflection tests inherit
both faults. A genuine frequency corpus is required; there is no offline shortcut.

### What this says about achievable accuracy

Rows 1 and 2 had 442 and 56 legal words. They are genuinely under-determined, and the model's
low confidence there (0.19 and 0.16) was the one thing it got right — it was paired with the
wrong ranking. **The target metric is top-3/top-5 recovery, not top-1**, for every row whose
`|Sᵢ|` is large. Only row 3 (`|S₃| = 7`) was ever determined enough to name outright, which is
exactly the row the model got right on both boards.

## What the prototype did not establish

The round-trip backtest (simulate a rational player → emit grid → reconstruct → measure
top-1 **and top-3** recovery per turn) is specified below but **was not run to completion**.
In Python each board costs ~6s because the belief set is 14,855 words, and the simulator on
top of that made a 20-board sweep impractical. It should be measured in Rust, where the same
work is ~50× cheaper. The penalty constants and `β = 1.5` are reasoned defaults validated on
one board, not fitted values.

Sequence the work so the backtest is not measuring the wrong thing: **fix the word prior
first**. Tuning `β` or the penalty constants against a broken familiarity term would fit those
constants to compensate for it. Note also that a simulated rational player is a best case —
it is generated by the same kind of model doing the reconstructing. The two human boards here
are worth more as validation than a large synthetic sweep, and more ground-truth boards
(guesses plus grid) are the most valuable test data to collect.

## Implementation plan

### Data — prerequisite, not an enhancement

- `wordle-data/word_frequency.txt` (new, generated by `scripts/update_wordle_data.py` from a
  frequency corpus such as `wordfreq`, emitting `word zipf` for all 14,855 allowed guesses).
  The generator is Python, so the Rust crate stays dependency-free — it only reads the file.

  This was originally scoped as the highest-value improvement; ground-truth validation
  promoted it to a **prerequisite**. Without it the model does not merely tie in the endgame,
  it actively demotes the correct words (`tears` #1 → #24). Ship this before the beam search.

- Retire the hand-written opener table. Derive turn-1 priors from frequency × entropy, or
  from the user's own history once that exists. The hand-made table measurably hurt.

- Clamp the frequency term (e.g. Zipf capped at ~5.0 before the exponent) so proper-noun
  artefacts like `roger` cannot override a meaningful entropy gap.

### Rust
- `src/reconstruct.rs` — share-text parsing, belief-set tracking, constraint penalties,
  stratified beam, marginalisation.
- `src/rank.rs` — extract a prepared `GuessContext { candidate_set, answer_probability }`.
  `evaluate_information_guess` currently rebuilds a `HashMap` and `HashSet` per call
  (`src/rank.rs:113`); at ~350 guesses per beam state that is ~1M wasted inserts per turn.
  Existing callers keep their signature via a thin wrapper.
- `src/server.rs` — `POST /v1/reconstruct`.
- `src/bin/reconstruct_backtest.rs` — the round-trip metric, mirroring `src/bin/backtest.rs`.

Share-text parsing must handle `1,874` comma separators, `X/6` losses, the `*` hard-mode
marker, `⬛`/`⬜` dark/light, `🟧`/`🟦` high-contrast, and variation selectors.

### API

```json
POST /v1/reconstruct
{ "share_text": "Wordle 1,874 6/6\n⬛⬛⬛🟨🟨\n…",
  "answer": "gripe",          // optional; else resolved via puzzle_number/date
  "rationality": 1.5, "alternatives_limit": 5 }
```

Response per turn: `word`, `probability`, `row_candidates`, `alternatives[]`, `confidence`,
`rationale`. Errors: `invalid_share_text` (400), `unknown_answer` (409), `impossible_row` (422).

### Answer resolution

`past_solutions.json` lags by a day, so today's puzzle is never resolvable — 1,874 was not in
the data during this work. The grid alone does not identify the answer either: 365 of 2,354
candidate answers admit a legal guess for all six rows of this board. The user must supply it.
Ranking those 365 by total sequence likelihood is a plausible future feature, not v1.

### UX

Not a fourth mode tab — an import affordance on the board ("Paste a shared result"), because
the output maps exactly onto `submittedGuesses = [{word, statuses}]`
(`static/index.html:1406`). Reconstruct fills the board and Easy Mode, Post Game and Advanced
then work on it unchanged. That composition is the reason to build this inside Wirdle.

Flow: paste → parse → resolve or prompt for the answer → **reconstruction preview** showing
each row's inferred word over its real tile colours, a confidence chip, and an expander
listing ranked alternatives ("*246 other words fit this row*") → `Use this board` /
`Review this game`.

Confidence chips: **Certain** (`|Sᵢ| = 1`), **Confident** (≥ 0.6), **Likely** (≥ 0.3),
**Uncertain** (< 0.3, alternatives expanded by default). Row 1 will be Uncertain on most
boards. Never render a guess as fact.

Ground truth confirms this is the right shape for the UI, not a hedge. On the 4/6 board the
correct words for rows 1–2 sat at ranks 6 and 5 after the prior fix — present in a
five-alternative expander, absent from any single-word answer. An uncertain row should read
as "one of these", and the alternatives list is the actual product surface for early rows.

### Spoiler handling

Reconstruction reveals the answer by construction. When the answer comes from a lookup rather
than the user, gate behind the existing Advanced Analyses-style confirmation.

## Connection to existing code

`src/coach.rs` already models suboptimal human play — `ConstraintDiscipline::Miss`,
`MoveType::ConstraintMiss`. That is the same vocabulary the penalty term needs, and this board
is a live example: the turn-4 guess drops a known-present letter under any `p…` turn-3 line.
Reconstruction and coaching should share one definition of a constraint miss.
