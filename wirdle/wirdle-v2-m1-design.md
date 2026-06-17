# Wirdle v2 Milestone 1 Engineering Design

## Summary

Milestone 1 turns Wirdle from a default solver UI into a post-game coach while preserving the existing board recreation workflow and solver engine.

The implementation adds a shared coach analysis layer and one new HTTP endpoint, `POST /v1/coach`, for user-facing coaching output. In M1 the endpoint supports only post-game review. In M2 the same endpoint can support Easy Mode hints by adding a new request intent and hint-specific response field. The existing `POST /v1/solve` endpoint remains available for Advanced Analyses after an explicit spoiler warning.

This keeps the public API small:

- Keep `/v1/solve` for solver-grade Advanced Analyses.
- Add `/v1/coach` for all human-facing coaching modes.
- Do not add separate `/v1/review`, `/v1/hint`, or `/v1/share` endpoints.

## Goals

- Generate a deterministic post-game report for a completed board.
- Produce turn-by-turn labels, letter grades, strengths, misses, and coaching notes.
- Generate a concise game summary and share text with no guess words.
- Reject inconsistent boards with a repair-focused error before producing grades, hints, or share text.
- Move existing solver recommendations behind an Advanced Analyses warning.
- Structure the coach layer so M2 Easy Mode hints reuse the same board analysis and API envelope.

## Non-Goals

- No live hint ladder in M1.
- No considered guesses in M1.
- No account-backed history or learning over time.
- No automatic NYT import.
- No new solver-ranking endpoint beyond the existing `/v1/solve`.
- No exact candidate lists or entropy-style math in the default post-game report.
- No M1 UI controls for puzzle number, puzzle date, or Hard Mode.

## Resolved Product Decisions

- M1 does not ask for puzzle number or date. Share text uses the entered board result and grid without puzzle metadata.
- M1 report copy refers to turn numbers and strategy labels, not guess words. This avoids ambiguity because Wordle allows repeated guesses.
- M1 keeps `hard_mode` as an API field with a default of `false`, but does not expose a Hard Mode control in the UI. Easy Mode can introduce that control later when live hints or concrete guess help need to avoid unplayable Hard Mode suggestions.

## Current System

The repo currently has:

- Manual board input in `static/index.html`.
- A dependency-free Rust server in `src/server.rs`.
- Core Wordle primitives:
  - `Word` parsing and validation.
  - `LetterStatus` feedback.
  - `GuessInput`.
  - `filter_candidates` and `is_candidate_consistent`.
  - likely-answer ranking.
  - information-gain ranking.
  - past-solution weighting.
- HTTP routes:
  - `GET /`
  - `GET /v1/health`
  - `GET /v1/metadata`
  - `POST /v1/solve`

M1 should reuse those primitives instead of adding a separate coaching model.

## Architecture

```text
Browser board state
        |
        | POST /v1/coach { intent: "post_game_review", guesses, ... }
        v
HTTP parser in src/server.rs
        |
        v
Coach layer in src/coach.rs
        |
        +--> BoardAnalysis
        |       - completion state
        |       - consistency state
        |       - per-turn candidate counts
        |       - per-turn information value
        |       - constraint discipline
        |       - solve/probe classification
        |       - trap and duplicate-letter notes
        |
        +--> PostGameReport
        |       - turn reviews
        |       - summary
        |       - share text
        |
        v
JSON response for UI rendering
```

`BoardAnalysis` is the shared internal contract. Post Game uses it in M1. Easy Mode uses it in M2 to choose the next hint level without duplicating board validation, candidate filtering, constraint notes, or stage detection.

## API Surface

### Endpoint Decision

Add:

```text
POST /v1/coach
```

Do not add separate endpoints for report generation, hint generation, and share text. The endpoint is stateless and derives output from the submitted board plus optional UI/session context.

`POST /v1/solve` stays as the Advanced Analyses endpoint. The UI should call it only after the user confirms the spoiler warning.

### M1 Request

```json
{
  "intent": "post_game_review",
  "guesses": [
    {
      "word": "slate",
      "statuses": ["absent", "present", "absent", "correct", "absent"]
    }
  ]
}
```

Fields:

- `intent`: required. M1 accepts only `post_game_review`.
- `guesses`: required. Same shape as `/v1/solve`.
- `hard_mode`: optional, default `false`. Kept for API compatibility, but the M1 UI does not expose it.
- `share_context`: optional and reserved for later. The M1 UI does not send puzzle number or date.

### M2-Compatible Request Extension

M2 can extend the same endpoint:

```json
{
  "intent": "easy_hint",
  "guesses": [
    {
      "word": "slate",
      "statuses": ["absent", "present", "absent", "correct", "absent"]
    }
  ],
  "hard_mode": false,
  "hint_request": {
    "requested_level": 2,
    "explain_current": false,
    "confirmed_spoiler": false
  },
  "session_context": {
    "highest_hint_level_used": 1
  }
}
```

M1 does not implement `easy_hint`, but the envelope reserves obvious slots for it. This avoids adding a future `/v1/hint` endpoint and makes share generation consume the same session context later.

### M1 Success Response

```json
{
  "intent": "post_game_review",
  "board": {
    "state": "solved",
    "turns": 4,
    "remaining_candidates": 1
  },
  "post_game": {
    "grades": ["B+", "B", "A-", "A"],
    "turns": [
      {
        "turn": 1,
        "label": "Balanced Opener",
        "grade": "B+",
        "move_type": "probe",
        "information": "solid",
        "constraint_discipline": "clean",
        "did_well": "It tested a balanced mix of useful letters.",
        "missed": "It did not narrow the ending much.",
        "summary": "A sound opening probe."
      }
    ],
    "summary": {
      "best_move_turn": 4,
      "most_questionable_turn": 2,
      "biggest_information_gain_turn": 3,
      "best_recovery_turn": null,
      "missed_opportunity": "Turn 2 could have spent more effort on the main pattern uncertainty.",
      "lesson": "When a word family is forming, switch from one-by-one guesses to a pattern splitter."
    }
  },
  "share": {
    "text": "Wordle 4/6\n...\n\nWirdle: Post Game Mode\nGrades: B+ / B / A- / A\nCoach: best move was a pattern splitter. Lesson: avoid guessing one-by-one in word-family traps.\n\nwirdle.app",
    "contains_guess_words": false
  }
}
```

The response may include internal category names such as `solid` or `clean`, but it should not include entropy, expected remaining answers, percentile ranks, full candidate lists, exact recommended words, or exact alternative guesses in the default Post Game payload.

### Error Responses

Malformed requests return `400 Bad Request`:

```json
{
  "error": "invalid_request",
  "message": "guess statuses must contain five values"
}
```

Incomplete post-game boards return `422 Unprocessable Entity`:

```json
{
  "error": "board_incomplete",
  "message": "Finish the Wordle board before reviewing your game."
}
```

Inconsistent boards return `422 Unprocessable Entity`:

```json
{
  "error": "board_inconsistent",
  "message": "This board does not match any possible Wordle answer. Check your tile colors and update the board before getting hints or analysis."
}
```

For M1, the UI should show these messages near the board and should not render report cards or share text.

## Domain Model

Add `src/coach.rs` and export it from `src/lib.rs`.

Core structs:

```rust
pub enum CoachIntent {
    PostGameReview,
    EasyHint,
}

pub enum BoardState {
    InProgress,
    Solved { turn: usize },
    Lost,
}

pub struct CoachRequest {
    pub intent: CoachIntent,
    pub guesses: Vec<GuessInput>,
    pub hard_mode: bool,
    pub share_context: Option<ShareContext>,
}

pub struct BoardAnalysis {
    pub state: BoardState,
    pub turns: Vec<TurnAnalysis>,
    pub final_remaining_candidates: usize,
}

pub struct TurnAnalysis {
    pub turn_index: usize,
    pub guess: Word,
    pub candidates_before: usize,
    pub candidates_after: usize,
    pub information_bucket: InformationBucket,
    pub constraint_discipline: ConstraintDiscipline,
    pub move_type: MoveType,
    pub trap_risk: TrapRisk,
    pub duplicate_letter_note: DuplicateLetterNote,
}

pub struct PostGameReport {
    pub turns: Vec<TurnReview>,
    pub summary: GameSummary,
    pub share_text: String,
}
```

These names are illustrative; implementation can collapse small enums if that fits the current code style better.

## Board Completion And Consistency

Post Game requires a completed board:

- Solved: any submitted row has all five statuses `correct`.
- Lost: six submitted rows and none are all-correct.
- Incomplete: fewer than six submitted rows and no all-correct row.

Validation order:

1. Parse all guesses and statuses.
2. Reject malformed words/statuses.
3. Build prefix candidate pools for each turn.
4. If any prefix has zero candidates, return `board_inconsistent`.
5. For `post_game_review`, reject `InProgress` as `board_incomplete`.
6. Generate report only after the board is complete and consistent.

If a solved row appears before later rows, M1 should not silently ignore the extra rows. Treat rows after a solve as invalid request data because a real Wordle game ends at the solve.

## Shared Board Analysis

For each turn, compute from the state before the guess:

- `candidates_before`: candidate solutions matching all prior feedback.
- `candidates_after`: candidate solutions after adding current feedback.
- `is_candidate_solve`: whether the actual guess was a possible answer before the turn.
- `respects_known_info`: whether `is_candidate_consistent(actual_guess, prior_guesses)` is true.
- `information_value`: how sharply the guess split `candidates_before`.
- `remaining_guesses`: `6 - turn_index`.
- `stage`: opener, middle game, endgame, or final guess.
- `duplicate_letters`: whether repeated letters were present and whether they were useful.
- `trap_risk`: whether the remaining candidates form a tight family.

Refactor `src/rank.rs` to expose a helper that can score one guess against a candidate pool. Today `rank_information_guesses` computes these values while ranking every legal word. The coach needs the same metrics for the actual played guess without forcing response code to inspect the full Advanced Analyses ranking.

Suggested helper:

```rust
pub fn evaluate_information_guess(
    guess: Word,
    candidates: &[Word],
    likely_answers: &[LikelyAnswer],
    past: &PastSolutionIndex,
) -> InformationGuess
```

`rank_information_guesses` can call this helper internally.

## Turn Review Heuristics

M1 grades are deterministic heuristic buckets, not an ML model.

### Move Type

- `probe`: not a possible answer, useful for splitting candidates.
- `solve_attempt`: possible answer before the turn.
- `forced_solve`: possible answer with few guesses or few candidates left.
- `constraint_miss`: does not respect known information.

### Information Bucket

Use candidate reduction and information metrics internally, then translate them into language:

- `sharp`: greatly narrows the puzzle.
- `solid`: meaningfully reduces uncertainty.
- `modest`: helps but leaves several similar answers alive.
- `low`: repeats information or barely changes the answer pool.

No numeric metric should appear in the default report text.

### Constraint Discipline

Use prior feedback to detect:

- ignored green positions.
- misplaced known yellow letters.
- reused gray letters when duplicate-letter feedback does not justify them.
- hard-mode violations when `hard_mode` is true.

In normal mode, a constraint miss is not illegal, but the coach should still explain why it was strategically costly.

### Labels

Choose one primary label per turn. M1 should support at least:

- Balanced Opener
- Vowel-Heavy Opener
- Constraint Builder
- Pattern Splitter
- Candidate Solve
- Trap Breaker
- Risky Direct Solve
- Duplicate-Letter Test
- Low-Information Repeat
- Constraint Miss

Label priority:

1. Constraint Miss, if known information was ignored.
2. Candidate Solve or Risky Direct Solve, if the guess was a possible answer.
3. Trap Breaker, if it tested a tight word-family pattern.
4. Duplicate-Letter Test, if repeated letters were strategically relevant.
5. Pattern Splitter or Constraint Builder, based on information value and stage.
6. Balanced Opener or Vowel-Heavy Opener for turn 1.
7. Low-Information Repeat, if no stronger label applies and the information bucket is low.

### Grades

Grade from a weighted score derived from:

- constraint discipline.
- information bucket.
- stage appropriateness.
- whether solve pressure called for an answer-shaped guess.
- whether the guess was a plausible human word or an obscure legal probe.
- duplicate-letter usefulness.
- trap handling.

Suggested mapping:

- `A`: excellent move for the turn context.
- `B`: reasonable and useful.
- `C`: understandable but missed a clearer strategic need.
- `D`: poor information or poor constraint discipline.
- `F`: severe contradiction with known information.

Use plus/minus for near-boundary cases. Keep grading stable and explainable rather than chasing tiny rank differences.

## Game Summary

Generate:

- best move.
- most questionable move.
- biggest information gain.
- best recovery.
- missed opportunity, if clear.
- one lesson for next time.

Selection rules:

- Best move: highest grade, breaking ties by information bucket and later-stage importance.
- Most questionable move: lowest grade, but avoid calling a forced final guess questionable if it was the only realistic play.
- Biggest information gain: largest candidate-pool reduction or strongest information bucket.
- Best recovery: a strong move immediately after a weaker move or after a board state with high trap risk.
- Missed opportunity: include only when a turn has a clear issue. Do not invent one for a clean game.
- Lesson: choose the highest-impact repeated theme, such as constraint discipline, switching from probing to solving, trap breaking, or duplicate-letter caution.

The summary should refer to turn numbers and strategy labels, not exact guess words. Share text must not include guess words at all.

## Share Text

Generate share text server-side as part of the coach response so the no-guess-words rule is enforced in one place.

M1 share text includes:

- official-style Wordle result line when possible.
- official-style grid derived from statuses.
- `Wirdle: Post Game Mode`.
- grades.
- one concise coach sentence.
- `wirdle.app`.

Rules:

- Never include actual guess words.
- Never include answer words.
- Never include candidate lists.
- Never include exact recommendations.
- Do not generate share text for incomplete or inconsistent boards.
- Do not include puzzle number or date in M1 because the UI does not collect them.

Implementation detail: because the repo defaults to ASCII source, keep Unicode square rendering isolated in one small function and test it directly. The rest of the coach logic can stay ASCII-only.

## UI Design

### Main Screen

Keep the Wordle board as the central input.

Add a mode/action area separate from board controls:

- Post Game panel:
  - primary button: `Review my game`.
  - disabled until the board is complete.
  - renders summary, turn timeline, and share button.
  - report text uses turn numbers and strategy labels rather than guess words.
- Advanced Analyses panel:
  - entry button with spoiler warning.
  - calls `/v1/solve` only after confirmation.
- Easy Mode:
  - do not expose a dead hint button in M1.
  - structure JS state so M2 can add `Get a hint` without changing the board model.
- Hard Mode:
  - do not expose a Hard Mode toggle in M1.
  - send the default `hard_mode: false` value if the client includes the field at all.

### Report Layout

Default report:

1. Overall summary.
2. Turn timeline.
3. Share button.

Avoid dense metric tables. Advanced metrics belong behind Advanced Analyses.

### Existing Solver UI

The current Power Words and Answers UI becomes Advanced Analyses. M1 should prevent solver-style results from being the default first screen and should not fetch `/v1/solve` until the user confirms the warning.

## Implementation Plan

1. Add `src/coach.rs`.
   - Define board state, analysis structs, report structs, and share context.
   - Build board completion and consistency validation.
2. Refactor `src/rank.rs`.
   - Extract `evaluate_information_guess`.
   - Keep `rank_information_guesses` behavior unchanged.
3. Implement `analyze_board`.
   - Compute candidate pools per prefix.
   - Compute turn-level features.
4. Implement post-game report generation.
   - Labels.
   - grades.
   - turn text.
   - game summary.
5. Implement share text generation.
   - Result line.
   - grid.
   - grades line.
   - coach sentence.
6. Add `POST /v1/coach` to `src/server.rs`.
   - Reuse the existing request parsing approach.
   - Return structured errors for incomplete and inconsistent boards.
7. Update `static/index.html`.
   - Add mode/action area.
   - Add Review my game flow.
   - Add report renderer and copy-share behavior.
   - Gate existing solver UI behind Advanced Analyses warning.
   - Do not add puzzle metadata inputs or a Hard Mode toggle in M1.
8. Add focused tests.

## Testing Plan

Rust unit tests:

- solved board is accepted.
- six-row loss is accepted.
- incomplete board returns `board_incomplete`.
- inconsistent board returns `board_inconsistent`.
- extra rows after solve are rejected.
- duplicate-letter feedback remains consistent with existing solver behavior.
- `evaluate_information_guess` matches the same values returned through `rank_information_guesses`.
- labels and grades are deterministic for fixture games.
- share text includes grades and grid.
- share text does not include any submitted guess words.

HTTP tests:

- `POST /v1/coach` parses a valid post-game request.
- `POST /v1/coach` rejects malformed statuses.
- `POST /v1/coach` returns `422` for incomplete and inconsistent boards.
- `POST /v1/solve` behavior is unchanged.

UI checks:

- Review button is disabled until a solved board or six submitted rows.
- Inconsistent board error appears near the board.
- Report renders summary before the turn timeline.
- Share button copies text with no guess words.
- Share text omits puzzle number and date.
- No Hard Mode control is shown in the M1 UI.
- Advanced Analyses warning appears before solver recommendations are shown.

## M2 Compatibility

M2 Easy Mode should reuse:

- `CoachRequest`.
- `BoardAnalysis`.
- board consistency validation.
- stage detection.
- constraint-discipline notes.
- trap-risk analysis.
- duplicate-letter analysis.
- share generation.

M2 should add:

- `CoachIntent::EasyHint`.
- `HintRequest`.
- `HintResponse`.
- hint level selection.
- spoiler confirmation flags for Level 5 and Level 6.
- client-side session tracking for highest hint level used.

The server should remain stateless. The browser can pass `session_context.highest_hint_level_used` back to `/v1/coach` when generating M2 share text. That avoids a session API and keeps M1's share machinery useful for M2.

## Risks And Mitigations

- Heuristic quality may feel generic.
  - Mitigation: build report text from concrete turn features and add fixture tests for representative games.
- Duplicate-letter reasoning is easy to explain poorly.
  - Mitigation: rely on exact feedback simulation for consistency and keep duplicate-letter notes conservative.
- Manual JSON handling can get brittle as responses grow.
  - Mitigation: keep request parsing narrow in M1 and centralize JSON escaping/building helpers. Revisit `serde_json` only if M2 response complexity makes manual code risky.
- Advanced Analyses gating changes the default experience.
  - Mitigation: preserve existing solver UI and route, but make the warning the entry point.
- Letter grades can imply false precision.
  - Mitigation: grade by broad buckets and pair every grade with plain-language reasoning.

## Open Questions

- None for M1.
