# Wirdle v2 Milestone 2 Engineering Design

## Summary

Milestone 2 adds Easy Mode, a live hint ladder for players who are stuck while playing Wordle. It reuses the Milestone 1 board entry workflow, `POST /v1/coach` endpoint, and `src/coach.rs` board analysis layer.

The main implementation change is to turn the currently reserved `CoachIntent::EasyHint` path into a real response branch. The server remains stateless: the browser sends the submitted board, requested hint escalation, and session hint context on each request. The browser owns UI state such as the highest hint level used, the currently displayed hint, and whether the user confirmed spoilery levels.

Easy Mode must preserve discovery by default:

- Early hints should usually be Level 1 or Level 2.
- Level 3 and Level 4 require explicit stronger-hint actions.
- Level 5 and Level 6 require confirmation because they can reduce or end the puzzle.
- Default hint copy should not expose candidate lists, exact words, entropy, expected remaining, or solver ranks.

## Goals

- Add an Easy Mode tab and `Get a hint` action.
- Implement the hint ladder from Gentle Nudge through Answer Reveal semantics.
- Reuse `analyze_board` for consistency checks, candidate pools, constraint notes, stage detection, trap risk, duplicate-letter notes, and information buckets.
- Track highest hint level used in client state.
- Include Easy Mode hint usage in share text, including in-progress Easy Mode shares.
- Add a Hard Mode toggle so live hints respect the player's Wordle mode.
- Keep Advanced Analyses as the only place that shows dense solver output by default.
- Keep the server dependency-free and consistent with the current manual JSON response style.

## Non-Goals

- No account-backed hint history.
- No automatic NYT board import.
- No browser extension.
- No full candidate lists in Easy Mode.
- No exact live guess recommendation before explicit Level 5 confirmation.
- No new `/v1/hint` endpoint.
- No persistent server-side session API.
- No considered-guesses flow. That remains Milestone 4.

## Resolved Product Decisions

- M2 accepts the Level 6 answer reveal constraint: reveal only when the entered board uniquely determines one compatible candidate. Do not add an official current-puzzle answer source in M2.
- Easy Mode should expose a Hard Mode toggle. When enabled, Level 5 guess help must only show words that are playable under the known constraints.
- Easy Mode share text can be copied before the puzzle is complete. This supports sharing that the player is currently stuck without revealing guess words, recommendations, candidates, or the answer.
- Easy Mode is the default selected tab and appears left of Post Game and Advanced Analyses. This makes live coaching the primary first-screen action while keeping Post Game and solver-style analysis available.

## Current System

M1 already provides most of the necessary foundation:

- `src/coach.rs`
  - `CoachIntent::EasyHint` exists but currently returns `unsupported_intent`.
  - `CoachRequest` contains `intent`, `guesses`, and `hard_mode`.
  - `analyze_board` validates board consistency and computes `TurnAnalysis`.
  - `BoardAnalysis` includes board state, turn features, and final remaining candidate count.
  - Post Game share text is generated server-side.
- `src/rank.rs`
  - `evaluate_information_guess` can score a single played or candidate guess.
  - `rank_likely_answers` and `rank_information_guesses` can support stronger hint levels.
- `src/server.rs`
  - `POST /v1/coach` parses the shared coach envelope.
  - Errors are mapped to `400` or `422`.
  - JSON parsing and rendering are manual.
- `static/index.html`
  - Board recreation is central and works for Post Game and Advanced Analyses.
  - The mode tabs currently include Post Game and Advanced Analyses only.
  - Post Game calls `/v1/coach`.
  - Advanced Analyses calls `/v1/solve` only after a spoiler warning.

M2 should extend this shape instead of introducing a parallel hint subsystem.

## Architecture

```text
Browser board state
        |
        | POST /v1/coach
        | intent: "easy_hint"
        | hint_request + session_context
        v
HTTP parser in src/server.rs
        |
        v
Coach layer in src/coach.rs
        |
        +--> analyze_board
        |       - board consistency
        |       - current candidate pool
        |       - stage
        |       - constraint discipline
        |       - trap risk
        |       - duplicate-letter notes
        |
        +--> build_easy_hint
        |       - choose/validate hint level
        |       - derive spoiler risk
        |       - produce one hint card
        |       - produce optional stronger-hint warning
        |       - update share-safe usage summary
        |
        v
JSON response rendered by Easy Mode panel
```

`/v1/solve` remains unchanged and should not be called by Easy Mode. If Easy Mode needs likely answers or information scores internally, it should call rank helpers from `src/coach.rs`, not route through the HTTP solver response.

## API Surface

### Endpoint

Continue using:

```text
POST /v1/coach
```

Accepted intents after M2:

- `post_game_review`
- `easy_hint`

### Easy Hint Request

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
    "highest_hint_level_used": 1,
    "hint_levels_used": [1]
  }
}
```

Fields:

- `hint_request.requested_level`: optional. If missing, the server chooses the lowest useful first hint for the board.
- `hint_request.explain_current`: optional, default `false`. Expands the current level without escalating.
- `hint_request.confirmed_spoiler`: optional, default `false`. Required for Level 5 and Level 6.
- `session_context.highest_hint_level_used`: optional, default `0`. Used for share text and escalation guardrails.
- `session_context.hint_levels_used`: optional. Used only for share text labels and UI continuity.

The server should clamp requested levels to `1..=6`. Invalid non-numeric or malformed objects should return `400 invalid_request`.

### Easy Hint Success Response

```json
{
  "intent": "easy_hint",
  "board": {
    "state": "in_progress",
    "turns": 2,
    "remaining_candidates": 18
  },
  "easy_hint": {
    "level": 2,
    "label": "Next-Move Strategy",
    "spoiler_risk": "low_medium",
    "message": "Try to place the yellow vowel while learning about the first position.",
    "rationale": "Your known letter is useful, but the first position is still doing most of the work.",
    "next_action_label": "A little more",
    "requires_confirmation_for_next": false,
    "revealed_words": [],
    "share_summary": {
      "highest_hint_level_used": 2,
      "hint_labels_used": ["Gentle Nudge", "Next-Move Strategy"]
    }
  },
  "post_game": null,
  "share": {
    "text": "Wordle ?/6\n...\n\nWirdle: Easy Mode\nHints: Gentle Nudge, Next-Move Strategy\nHighest hint: Level 2\nCoach: helped me use the clues without giving it away.\n\nwirdle.onrender.com",
    "contains_guess_words": false
  }
}
```

Notes:

- `easy_hint` is `null` for `post_game_review`.
- `post_game` is `null` for `easy_hint`.
- `revealed_words` is empty for Levels 1-4.
- Level 5 may include one to three revealed words after confirmation.
- Level 6 may include a single reveal only when M2 can honestly identify one answer-compatible word. See Answer Reveal Constraint below.
- `share.text` for in-progress games uses `Wordle ?/6` plus the entered grid. Once the board is solved or lost, it uses the normal result line.

### Error Responses

Reuse M1 error behavior:

```json
{
  "error": "board_inconsistent",
  "message": "This board does not match any possible Wordle answer. Check your tile colors and update the board before getting hints or analysis."
}
```

Additional M2 errors:

```json
{
  "error": "spoiler_confirmation_required",
  "message": "This hint can reveal answer-like words. Confirm that you want a stronger hint."
}
```

```json
{
  "error": "answer_reveal_unavailable",
  "message": "Wirdle cannot identify one exact answer from this board yet. Ask for strong guess help instead."
}
```

`spoiler_confirmation_required` should be `422`, because the request shape is valid but the user has not confirmed escalation.

## Domain Model

Extend `src/coach.rs`:

```rust
pub struct CoachRequest {
    pub intent: CoachIntent,
    pub guesses: Vec<GuessInput>,
    pub hard_mode: bool,
    pub hint_request: Option<HintRequest>,
    pub session_context: Option<SessionContext>,
}

pub struct HintRequest {
    pub requested_level: Option<u8>,
    pub explain_current: bool,
    pub confirmed_spoiler: bool,
}

pub struct SessionContext {
    pub highest_hint_level_used: u8,
    pub hint_levels_used: Vec<u8>,
}

pub struct CoachResponse {
    pub intent: CoachIntent,
    pub board: BoardSummary,
    pub post_game: Option<PostGameReport>,
    pub easy_hint: Option<HintResponse>,
    pub share: Option<ShareOutput>,
}

pub struct HintResponse {
    pub level: HintLevel,
    pub label: &'static str,
    pub spoiler_risk: SpoilerRisk,
    pub message: String,
    pub rationale: Option<String>,
    pub next_action_label: Option<&'static str>,
    pub requires_confirmation_for_next: bool,
    pub revealed_words: Vec<Word>,
    pub share_summary: HintShareSummary,
}
```

Suggested enums:

```rust
pub enum HintLevel {
    GentleNudge = 1,
    NextMoveStrategy = 2,
    UsefulLetter = 3,
    Pattern = 4,
    StrongGuessHelp = 5,
    AnswerReveal = 6,
}

pub enum SpoilerRisk {
    Low,
    LowMedium,
    MediumHigh,
    High,
    VeryHigh,
    Complete,
}
```

Keep the enum string renderers local to `src/coach.rs`, matching the existing `as_str` pattern for M1 enums.

## Board Validation

Easy Mode accepts in-progress boards, solved boards, and six-row losses, but behavior differs by state:

- `InProgress`: generate hints.
- `Solved` or `Lost`: do not generate new live hints. Return a friendly `game_finished` error or a response with no `easy_hint` and a UI message directing the user to Post Game.
- Any inconsistent prefix: return `board_inconsistent`.
- Any row after a solved row: return `invalid_request`, matching M1.

Unlike Post Game, Easy Mode must not reject an incomplete board merely because the game is in progress.

## Hint Level Selection

### First Hint

If `requested_level` is missing or lower than `1`, choose the lowest useful hint:

- Turn 0 with no submitted guesses: Level 1 opener guidance.
- Turn 1-3 with broad candidate pool: Level 1.
- Board with clear forgotten clue or constraint risk: Level 1.
- Board with clean constraints but obvious strategic uncertainty: Level 2.
- Never auto-start above Level 2.

### Stronger Hint

When the user asks for a stronger hint:

- Requested level is normally `highest_hint_level_used + 1`.
- The server may downgrade Level 3 or Level 4 to Level 2 if the board is too early and no useful letter or pattern hint can be given without excessive spoilage.
- The server must not auto-upgrade into Level 5 or Level 6.
- Level 5 and Level 6 require `confirmed_spoiler: true`.

### Explain Current

`explain_current: true` should keep the same level and add or replace `rationale` with more detail. It should not reveal stronger hint material.

## Hint Generation

All hint copy is deterministic and built from `BoardAnalysis` plus current candidates. Randomness is unnecessary and would make tests brittle.

### Shared Helpers

Add helpers in `src/coach.rs`:

- `current_candidates(solver, guesses) -> Vec<Word>`
- `known_green_positions(guesses) -> Vec<(usize, u8)>`
- `known_yellow_letters(guesses) -> Vec<KnownYellow>`
- `known_absent_letters(guesses) -> Vec<u8>`
- `important_unknown_positions(candidates) -> Vec<usize>`
- `useful_remaining_letters(candidates, guesses) -> Vec<u8>`
- `pattern_hint(candidates, guesses) -> Option<String>`
- `human_like_guess_options(solver, candidates, guesses, hard_mode) -> Vec<HintGuessOption>`

Prefer exact feedback and candidate filtering over ad hoc inference where possible. Small, copy-focused helpers may use the already computed `TurnAnalysis` fields.

### Level 1: Gentle Nudge

Purpose: remind the player what kind of thinking to do next.

Selection priority:

1. If a previous turn had a constraint miss risk, point to the known clue.
2. If there is an unplaced yellow, remind the user to place it.
3. If there is a green, remind the user to preserve it.
4. If vowels are mostly exhausted, nudge toward consonants.
5. If no guesses have been submitted, suggest a broad opener.
6. Otherwise name the stage: narrowing turn, solve attempt, or trap check.

No words, no candidate counts, no letter lists beyond letters already visible on the board.

### Level 2: Next-Move Strategy

Purpose: name the kind of guess that would help without naming a word.

Examples generated from state:

- Plausible answer that keeps a green and tests new consonants.
- Probe that places a yellow and checks an unknown position.
- Pattern splitter when many candidates share four positions.
- Direct solve when candidate count is very small and guesses are running out.

No exact guess words. It may mention positions and already-known letters.

### Level 3: Useful Letter Hint

Purpose: suggest useful letters to test.

Implementation:

- Count letter frequency across current candidates, excluding confirmed positions and already ruled-out information.
- Prefer common letters in candidate answers over obscure allowed-guess utility.
- Return up to three letters.
- Avoid suggesting a letter that is only useful in one obscure candidate unless the pool is already small.

This level may expose letters not already known, so it should only appear after explicit escalation.

### Level 4: Pattern Hint

Purpose: show partial answer structure.

Implementation:

- Start with five underscores.
- Fill green positions.
- Optionally fill one high-confidence shared position across candidates if doing so does not reveal the whole answer.
- Mention common endings or families when many candidates share a suffix or prefix.
- Do not include exact answer words.

If the pattern would reveal all five letters, require Level 6 instead.

### Level 5: Strong Guess Help

Purpose: help choose between plausible guesses while preserving a little agency.

Confirmation is required.

Implementation:

- Build answer-shaped options from current candidates.
- Score options with a human-like blend:
  - respects known info or hard-mode constraints.
  - plausible remaining answer.
  - common/reasonable word.
  - useful information score.
  - not known as a recent past solution unless repeats are otherwise plausible.
- Return one to three `revealed_words`.
- Prefer candidates over non-answer probes unless a probe is clearly needed and there are enough guesses left.

The message should explain why the options differ, not just rank them.

### Level 6: Answer Reveal Constraint

The current M1 data model does not know today's official answer. It only knows words compatible with the user's entered feedback and historical answer weighting. The PRD also says automatic NYT import is not part of the v2 MVP.

Therefore M2 should use this rule:

- If the compatible candidate pool has exactly one word, Level 6 can reveal that word as the answer implied by the board.
- If the pool has multiple words, return `answer_reveal_unavailable` and direct the user to Level 5 strong guess help.
- Do not claim that a likely-answer rank is the official answer.

This keeps the product honest. A future milestone can add an explicit current-puzzle answer source if product wants literal answer reveal before the board narrows to one candidate.

## Human-Like Ranking For Hints

Easy Mode should not simply reuse the top `rank_information_guesses` row. Add a small `HintGuessOption` scoring function:

```rust
struct HintGuessOption {
    word: Word,
    is_candidate: bool,
    information: InformationGuess,
    human_score: i32,
    explanation: String,
}
```

Score inputs:

- Candidate answer: strong positive, especially late.
- Respects hard mode: required when `hard_mode` is true.
- Respects known info: strong positive in all modes.
- Information bucket: positive, but not enough to make obscure probes dominate.
- Common answer-shaped word: positive. In M2 this can be approximated by candidate membership and past-solution weighting.
- Recent past solution: negative, matching current solver policy.
- Obscure allowed-only word: negative unless Level 5 explicitly needs a probe.

This ranking is only for hint wording. Advanced Analyses continues to show solver-grade rankings.

## Share Text

M2 should extend server-side share generation so the no-guess-words rule stays centralized.

Easy Mode share includes:

- official-style result line if solved/lost, otherwise `Wordle ?/6`.
- grid from entered statuses.
- `Wirdle: Easy Mode`.
- hint labels used.
- highest hint level used.
- one coaching sentence.
- `wirdle.onrender.com`.

In-progress Easy Mode share should be available directly from the Easy Mode panel after at least one hint. It should use the entered grid and `Wordle ?/6`, making it safe for a player to share that they are stuck before finishing the puzzle.

Combined Easy Mode plus Post Game share should be generated when a completed board has session context indicating hints were used and the user runs Post Game review. The Post Game response can include:

- `Wirdle: Easy Mode + Post Game Mode`.
- hint labels used.
- highest hint level.
- grades.
- one Post Game lesson.

Rules:

- Do not include exact guess words from Level 5 in share text.
- Do not include the Level 6 answer in share text.
- Do not include candidate lists.
- Do not include recommended words.
- `contains_guess_words` should remain `false` for submitted guesses. M2 should also check revealed hint words before returning share text.

## UI Design

### Mode Tabs

Add Easy Mode as the leftmost tab and default selected panel:

- Easy Mode
- Post Game
- Advanced Analyses

Keep board controls unchanged. Mode panels should remain separate from board editing controls. On initial page load, the Easy Mode tab should have `aria-selected="true"`, and the Post Game and Advanced Analyses panels should be hidden until selected.

### Easy Mode Panel

Initial in-progress state:

- Primary button: `Get a hint`.
- Hint card area hidden until a hint is available.
- Small state text such as `Live help`.

After first hint:

- Show one hint card.
- Show `A little more` or next action label from the server.
- Show `Explain that` when `rationale` is available or can be requested.
- Show confirmation overlay before Level 5 and Level 6.

For solved/lost boards:

- Disable live hint action.
- Show a short message that the game is complete and Post Game can review it.

### Hard Mode Control

Add a Hard Mode toggle inside the Easy Mode panel, near the hint action. It should be mode-specific UI, not a global setup screen.

Behavior:

- Default is off.
- Send `hard_mode` on every Easy Mode `/v1/coach` request.
- When on, Level 5 `revealed_words` must respect hard-mode constraints.
- When on, lower-level hints should frame constraint reminders as required playable constraints rather than just strategic advice.
- Toggling Hard Mode should clear the current hint card and request a fresh hint when the user asks again.
- Post Game can continue to omit Hard Mode controls unless product later wants hard-mode grading.

### Client State

Add JS state:

```js
let latestHintId = 0;
let latestHintData = null;
let highestHintLevelUsed = 0;
let hintLevelsUsed = [];
let pendingHintLevel = null;
let easyModeHardMode = false;
```

Reset hint state when:

- a guess is added.
- a tile status changes.
- undo is clicked.
- reset is clicked.
- the Easy Mode Hard Mode toggle changes.

Do not reset hint state merely because the user switches tabs.

### Spoiler Confirmation

Reuse the existing overlay system:

- Level 5 warning: can reveal answer-like words and reduce the puzzle.
- Level 6 warning: can reveal the answer if the board uniquely determines it.

The confirmation action should retry the same `/v1/coach` request with `confirmed_spoiler: true`.

### Accessibility

- Hint card should use `aria-live="polite"`.
- Buttons must be disabled while a hint request is in flight.
- Confirmation overlay should reuse the current modal focus behavior.
- Level labels should be present in the card, but the UI should not present the ladder as a setup selector.

## Server Implementation Plan

1. Extend `CoachRequest`.
   - Add `hint_request` and `session_context`.
   - Keep defaults so existing Post Game requests continue to parse.
2. Extend `CoachResponse`.
   - Add `easy_hint: Option<HintResponse>`.
   - Update `coach_response_json`.
3. Extend manual JSON parsing in `src/server.rs`.
   - Parse nested `hint_request`.
   - Parse nested `session_context`.
   - Parse numeric arrays for `hint_levels_used`.
4. Implement `easy_hint(solver, request)`.
   - Call `analyze_board`.
   - Reject inconsistent boards through existing validation.
   - Reject solved/lost boards with `game_finished`.
   - Select/validate hint level.
   - Enforce confirmation for Levels 5-6.
   - Build hint response.
   - Build Easy Mode share text.
5. Add hint generation helpers in `src/coach.rs`.
6. Add human-like Level 5 option ranking.
7. Extend share generation for Easy Mode and combined Easy Mode plus Post Game.
8. Add tests before wiring the UI.

## UI Implementation Plan

1. Add Easy Mode tab markup and panel as the leftmost/default selected mode.
2. Add hint card CSS using existing panel and button styles.
3. Add Easy Mode JS state and reset hooks.
4. Implement `requestHint({ requestedLevel, explainCurrent, confirmedSpoiler })`.
5. Render `easy_hint` response.
6. Add stronger-hint and explain actions.
7. Add Level 5/6 confirmation overlay.
8. Add the Easy Mode Hard Mode toggle and pass it as `hard_mode`.
9. Add an Easy Mode share button once at least one hint has been shown.
10. Pass `session_context` into Post Game review so combined share can include hint usage.
11. Update the help overlay to mention Easy Mode.

## Testing Plan

Rust unit tests:

- `easy_hint` accepts an in-progress consistent board.
- `easy_hint` rejects inconsistent boards with `board_inconsistent`.
- `easy_hint` rejects rows after solve with `invalid_request`.
- `easy_hint` returns `game_finished` for solved/lost boards.
- first hint never starts above Level 2.
- stronger hint advances from Level 1 to Level 2.
- `explain_current` does not escalate the level.
- Level 5 requires spoiler confirmation.
- Level 6 requires spoiler confirmation.
- Level 6 reveals only when candidate count is one.
- Level 6 with multiple candidates returns `answer_reveal_unavailable`.
- Level 1 and Level 2 responses contain no revealed words.
- Level 3 returns useful letters but no words.
- Level 4 returns pattern text but no words.
- Level 5 revealed words do not appear in share text.
- Easy Mode share includes hint labels and highest level.
- in-progress Easy Mode share uses `Wordle ?/6` and the entered grid.
- Combined share includes both hint usage and Post Game grades.
- `contains_guess_words` remains false for submitted and revealed words.
- Hard Mode Level 5 options all respect known constraints.

HTTP tests:

- `POST /v1/coach` parses `easy_hint`.
- malformed `hint_request` returns `400 invalid_request`.
- missing `hint_request` still returns a first hint.
- Level 5 without confirmation returns `422 spoiler_confirmation_required`.
- `post_game_review` behavior remains unchanged.

UI checks:

- Easy Mode tab appears left of Post Game and Advanced Analyses.
- Easy Mode panel is selected and visible on initial page load.
- `Get a hint` works on in-progress boards.
- hint state clears after board edits.
- `A little more` requests the next level.
- `Explain that` does not increase highest hint level.
- Level 5 and Level 6 show confirmation overlays.
- Hard Mode toggle clears stale hints and affects the next hint request.
- Easy Mode share can be copied before the board is solved or lost.
- solved/lost boards direct the user to Post Game instead of giving live hints.
- Post Game share after hint use includes Easy Mode usage.
- Advanced Analyses still requires spoiler confirmation before `/v1/solve`.

## Risks And Mitigations

- Hint copy may feel generic.
  - Mitigation: generate from concrete board features first: yellow placement, green preservation, candidate family, stage, and duplicate-letter evidence.
- Manual JSON parsing becomes brittle.
  - Mitigation: add narrow helpers for nested objects and numeric arrays; keep request shape small. Revisit `serde_json` only if this continues to grow after M2.
- Strong hints may spoil too quickly.
  - Mitigation: require explicit confirmation for Level 5 and Level 6; never auto-start above Level 2.
- Answer reveal expectations may exceed available data.
  - Mitigation: reveal only when the board uniquely determines one candidate; otherwise offer strong guess help and avoid claiming official answer knowledge.
- Share text could leak revealed words.
  - Mitigation: generate share server-side and test against submitted guesses plus `revealed_words`.

## Open Questions

- None for M2.
