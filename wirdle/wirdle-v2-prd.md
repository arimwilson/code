# Wirdle v2 PRD

## Product Summary

Wirdle v2 is a Wordle coaching product that helps players become better over time without taking away the satisfying "aha" moment.

The product keeps the familiar Wirdle board-recreation workflow, but pivots the main experience away from solver outputs and toward post-game coaching, strategic reflection, and gentle live help.

The current solver-style experience remains available only under **Advanced Analyses**, with explicit spoiler warnings.

## Product Modes

Wirdle has three user-facing modes:

1. **Post Game**
   - Primary v2 experience.
   - Used after a player finishes the daily Wordle.
   - Provides turn-by-turn coaching, strategy lessons, and optional alternative-guess analysis.

2. **Easy Mode**
   - Second deliverable.
   - Used while playing the daily Wordle.
   - Provides escalating hints that preserve as much discovery as possible.

3. **Advanced Analyses**
   - Spoiler-gated continuation of current Wirdle.
   - Shows likely answers, Power Words, candidate-style analysis, and solver-grade detail.
   - Explicitly labeled as likely to reduce the challenge of the puzzle.

The product should avoid up-front mode configuration. Users should encounter natural actions such as "Review my game," "Get a hint," or "Advanced Analyses" rather than setup screens.

## Goals

- Help players become better at Wordle over time.
- Make strategic reasoning feel human, practical, and understandable.
- Preserve the puzzle's emotional payoff.
- Make post-game reflection useful enough to become a daily habit.
- Make sharing demonstrate how helpful Wirdle was without revealing the puzzle.
- Reuse the existing board input, feedback modeling, candidate filtering, and ranking infrastructure.

## Non-Goals

- Do not make the default experience a solver.
- Do not show full candidate lists in Post Game or Easy Mode by default.
- Do not recommend exact live guesses early in the hint ladder.
- Do not require a browser extension for v2 MVP.
- Do not automatically import from NYT Wordle for v2 MVP.
- Do not make users choose a coaching level before they understand the product.

## Primary User Flow

### Existing Board Recreation

Users manually recreate their Wordle board in Wirdle:

- Type a guess.
- Submit the guess.
- Tap tiles to mark yellow or green feedback.
- Leave gray tiles as absent.
- Undo adding a guess.
- Reset the board.

This should remain familiar to existing Wirdle users.

### Post Game Flow

1. User finishes today's Wordle.
2. User opens Wirdle and recreates the board.
3. User chooses **Review my game**.
4. Wirdle generates a coaching report.
5. User copies a shareable Wirdle summary.
6. After Milestone 4, user can optionally add guesses they considered but did not play and compare them against the actual turn context.

Post Game should require a completed board: either a solved row with all five letters green, or six submitted wrong guesses.

### Easy Mode Flow

1. User gets stuck while playing today's Wordle.
2. User recreates the current board state in Wirdle.
3. User chooses **Get a hint**.
4. Wirdle gives the next hint in a spoiler-aware hierarchy.
5. User can request a stronger hint if still stuck.
6. The hint ladder can eventually reveal the answer, but only after clear escalation.
7. User can later review how hints were used after the game.

### Advanced Analyses Flow

1. User opens **Advanced Analyses**.
2. Wirdle displays a spoiler warning.
3. User confirms they want solver-style output.
4. Wirdle shows the current advanced recommendations: likely answers, Power Words, and more detailed rankings.

## Human-Like Coaching Definition

Wirdle should prioritize human-likeness in this order:

1. Explanations that sound like human reasoning.
2. Common real words.
3. Guesses that respect known positions and known letters.
4. Plausible Wordle answers.

This ranking matters. Wirdle should feel like a thoughtful friend sitting next to the player, helping them reason through what they already know. A technically strong hint is not good enough if it feels alien, solver-like, or disconnected from how a human would naturally think about the board.

## Post Game MVP

Post Game is the first v2 deliverable.

### Required Features

#### Turn-by-Turn Review

For each submitted guess, show:

- Strategic label.
- Letter grade.
- What the guess did well.
- What it missed.
- Whether it respected known information.
- Whether it was more of a probe or a solve attempt.
- Whether it meaningfully reduced uncertainty.

Example labels:

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

#### Game Summary

The summary should identify:

- Best move.
- Most questionable move.
- Biggest information gain.
- Best recovery.
- Missed opportunity, if any.
- One lesson for next time.

The tone should feel like a concise coach, not a math report.

#### Advanced Metrics Hidden by Default

Post Game may use candidate counts, entropy, expected remaining answers, and percentile rankings internally, but the default report should translate them into strategy language.

Possible visible phrasing:

- "This narrowed the puzzle sharply."
- "This left several similar answers alive."
- "This was a strong information move for the stage of the game."
- "This was reasonable, but it missed a chance to test a more important position."

Avoid default phrasing like:

- "Expected remaining answers: 4.8."
- "Entropy: 3.14 bits."
- "Ranked 38th among 14,855 legal guesses."

Those belong in Advanced Analyses.

### Future Feature: Considered Guesses

Users can optionally add guesses they considered but did not play.

For each considered guess, Wirdle should explain:

- Whether it would have been legal.
- Whether it respected known clues.
- Whether it was more likely to be useful than the actual guess.
- Whether it was a better human-like move, a better information move, or neither.

The UI should treat this as optional reflection, not a required part of entering the game. This is lower priority than the Post Game MVP and Easy Mode hint ladder.

## Easy Mode MVP

Easy Mode is the second deliverable.

Easy Mode gives live help through a hint ladder. The user should remain in control of escalation.

### Hint Ladder

Hints should progress from broad strategy to direct answer reveal. The ladder is a maximum escalation path, not the expected journey. Most successful Easy Mode sessions should end after one to three hints.

Each level should be request-driven, with clear language that the next hint is stronger. The UI should avoid making users feel like they are clicking through numbered hints. Button copy should feel natural, such as:

- "Give me a hint"
- "A little more"
- "Show useful letters"
- "Show a pattern"
- "Help me choose"
- "Reveal answer"

#### Level 1: Gentle Nudge

Purpose: Tell the player what kind of thinking to do next and remind them of earned information they may be forgetting.

Spoiler risk: Low.

Examples:

- "Focus on placing your known yellow letter."
- "You have enough vowel information. Common consonants matter more now."
- "This looks like a narrowing turn, not a final-answer turn."
- "Be careful about reusing letters that have already been ruled out."
- "The yellow A cannot go in position 3."
- "You already know position 5 is E."
- "The next guess should account for the letters you have already confirmed."
- "A gray letter may still matter only if duplicate-letter feedback allows it."

Progression rule: This should be the default first hint in most situations. It should feel like a friend pointing back to what the player already knows.

#### Level 2: Next-Move Strategy

Purpose: Name the kind of guess that would help and identify the most important uncertainty without naming a word.

Spoiler risk: Low to medium.

Examples:

- "Look for a plausible answer that keeps the green letter and tests two new consonants."
- "Try to place the yellow vowel while learning about the first position."
- "A probe may be better than guessing one possible answer at a time."
- "With only two guesses left, choose something that can still be the answer."
- "The biggest uncertainty is the first consonant."
- "A common consonant is likely more useful than another vowel here."
- "The ending is doing most of the work in this pattern."
- "A repeated vowel is possible, but do not jump to it unless other options fail."

Progression rule: Use when the player understands the constraints but needs help deciding the purpose of the next move. This should still preserve the player's ability to find their own word.

#### Level 3: Useful Letter Hint

Purpose: Suggest letters worth testing.

Spoiler risk: Medium to high.

Examples:

- "Testing R, N, or C would be useful."
- "One of the useful remaining consonants is likely near the start."
- "A word that tests both R and P would answer a lot."
- "Do not ignore the possibility of a second E."

Progression rule: Use only after the player requests a stronger hint. Avoid this level by default in the first few hints most of the time.

#### Level 4: Pattern Hint

Purpose: Show partial structure.

Spoiler risk: High.

Examples:

- "A useful answer-shaped pattern is `_ A _ E _`."
- "You are close to a family like `_ O U N D`."
- "The likely structure has a common ending."

Progression rule: Use only after specific letter guidance, or when the player explicitly asks for a stronger hint.

#### Level 5: Strong Guess Help

Purpose: Help the player choose between plausible guesses. This level may start with multiple options and then allow the user to ask Wirdle to recommend one.

Spoiler risk: Very high.

Examples:

- "Here are a few plausible directions. They all respect what you know, but they test different uncertainties."
- "If you want one recommendation, this is the one I would play."
- "Recommended: [word]. It respects the known positions, tests the most important remaining uncertainty, and is still a plausible answer."

Implementation note: The share output should not include the words. The UI can show the words live only after clear escalation.

Progression rule: Require explicit confirmation that this may reduce the puzzle. Use when the user wants strong help but still wants some agency before the answer reveal.

#### Level 6: Answer Reveal

Purpose: End the ladder.

Spoiler risk: Complete.

Progression rule: Require explicit "Reveal answer" action after warning.

### Hint Progression Behavior

The product should not expose hint levels as setup choices. Instead:

- The first hint starts at the lowest useful level for the current board state.
- "Stronger hint" advances one level.
- "Explain that" can expand the current level without escalating.
- Early turns should strongly prefer levels 1-2.
- Level 3 should be reserved for genuinely stuck players.
- Levels 4-6 are knowingly spoilery and should feel like escalation, not the normal path.
- Level 5 and Level 6 require confirmation.
- Each game should record the highest hint level used for sharing and post-game reflection.

## Sharing

Sharing should optimize for showing how helpful Wirdle was as a coach.

Share text should include the official Wordle share text and grid, plus a concise Wirdle coaching summary. This lets a player use only Wirdle to generate the text they send to friends.

Share text must not include actual guess words. It may include:

- Puzzle number or date, if available from user-entered context.
- Solved or not solved.
- Number of Wordle guesses.
- Wirdle mode or modes used.
- Highest hint level used.
- Hint types used.
- Post-game coaching highlights.
- Guess quality distribution.
- Top coaching lesson.
- Whether the user improved a weak move after a hint.
- Whether considered guesses were reviewed.

### Example Post Game Share

```text
Wordle 1,829 4/6
⬛⬛🟨⬛🟩
⬛🟨⬛⬛🟩
🟨🟩🟩⬛🟩
🟩🟩🟩🟩🟩

Wirdle: Post Game Mode
Grades: B+ / B / A- / A
Coach: best move was a pattern splitter. Lesson: avoid guessing one-by-one in word-family traps.

wirdle.app
```

### Example Easy Mode Share

```text
Wordle 1,829 5/6
⬛⬛🟨⬛⬛
⬛🟨⬛⬛🟩
⬛🟩🟨⬛🟩
🟨🟩🟩⬛🟩
🟩🟩🟩🟩🟩

Wirdle: Easy Mode
Hints: Gentle Nudge, Next-Move Strategy
Highest hint: Level 2
Coach: helped me place what I already knew without giving it away.

wirdle.app
```

### Example Easy Mode + Post Game Share

```text
Wordle 1,829 5/6
⬛⬛🟨⬛⬛
⬛🟨⬛⬛🟩
⬛🟩🟨⬛🟩
🟨🟩🟩⬛🟩
🟩🟩🟩🟩🟩

Wirdle: Easy Mode + Post Game Mode
Hints: Gentle Nudge, Next-Move Strategy
Highest hint: Level 2
Grades: B / B+ / A- / B+ / A
Coach: my best move narrowed the pattern; next time I should switch to solving sooner.

wirdle.app
```

### Share Constraints

- Do not include guess words.
- Do not include the answer.
- Do not include candidate lists.
- Do not include exact recommended words.
- Include the official Wordle grid and result line when enough board information is available.
- It is okay to include hint types.
- It is okay to include grades, strategy labels, and lesson summaries.

## Advanced Analyses

Advanced Analyses contains the current solver-style Wirdle functionality.

It may show:

- Likely answers.
- Power Words.
- Candidate counts.
- Ranked guesses.
- Best information guesses.
- Best likely-answer guesses.
- Entropy and expected remaining values.
- Hard-mode filtered analyses.

Entry should include a warning:

> Advanced Analyses can reveal likely answers and reduce the challenge of today's Wordle.

Advanced Analyses should be visually and conceptually separate from Post Game and Easy Mode.

## Data and Ranking Requirements

The existing solver engine can power v2, but the product needs additional interpretation layers.

### Reused Infrastructure

- Word parsing and validation.
- Feedback evaluation.
- Candidate filtering.
- Likely-answer ranking.
- Information-gain ranking.
- Hard-mode legality checks.
- Past-solution weighting.

### New Interpretation Layer

The coach layer should derive:

- Strategy labels.
- Human-like guess grades.
- Constraint discipline score.
- Information value bucket.
- Solve-pressure assessment.
- Trap-risk assessment.
- Duplicate-letter assessment.
- Hint ladder output.
- Share summary.

### Human-Like Ranking

When the coach needs to evaluate, compare, or explain guesses in user-facing modes, it should prefer:

1. Human-readable reasoning that matches what a friend would say.
2. Common, recognizable words.
3. Guesses that obey known constraints.
4. Remaining plausible answer candidates.
5. High-information guesses.
6. Obscure legal guesses only in Advanced Analyses.

The highest mathematical score should not automatically become the default coaching recommendation.

## UX Requirements

### Main Screen

The main screen should continue to center on the Wordle-style board.

Board state editing should be visually separate from mode selection. The user should not have to hunt through board controls to switch between Post Game, Easy Mode, and Advanced Analyses.

Board controls:

- Reset.
- Undo.
- Submit guess.
- Tile color editing.

Mode selection area:

- Post Game.
- Easy Mode.
- Advanced Analyses.

Mode-specific primary actions, such as **Review my game** or **Get a hint**, should live near the selected mode's panel rather than inside the board editing controls.

### Post Game Report

The report should be scannable:

- Overall summary at top.
- Turn timeline below.
- Optional considered-guesses section after Milestone 4.
- Share button.

Avoid dense tables in the default view.

### Easy Mode Hints

Hints should feel lightweight:

- One clear hint at a time.
- A "stronger hint" action.
- An "explain" action.
- Clear warning before answer-like levels.

### Advanced Analyses

Advanced Analyses may preserve a denser solver UI with tabs, rankings, and detailed metrics.

## Success Metrics

### Product Metrics

- Percentage of users who complete a Post Game review after entering a board.
- Percentage of Post Game users who copy share text.
- Percentage of users who return for another daily review.
- Percentage of Easy Mode users whose highest hint remains Level 1-3, with most sessions ending at Level 1 or Level 2.
- Percentage of users who enter considered guesses after that feature launches.
- Advanced Analyses usage rate relative to Post Game and Easy Mode.

### Quality Metrics

- Users report that Wirdle helped them understand a better strategy.
- Users report that Easy Mode did not spoil the puzzle too early.
- Share text is understandable without revealing the puzzle.
- Generated lessons are specific to the actual game, not generic tips.

## Milestones

### Milestone 1: Post Game MVP

- Keep manual board recreation.
- Add Review my game action.
- Generate turn-by-turn grades and labels.
- Generate concise game summary.
- Add share text without guess words.
- Include the official Wordle result line and grid in share text.
- Move existing solver UI behind Advanced Analyses warning.

### Milestone 2: Easy Mode Hint Ladder

- Add Get a hint action.
- Implement all hint levels from Gentle Nudge through Answer Reveal.
- Track highest hint level used.
- Include hint usage in share text.

### Milestone 3: Learning Over Time

- Track recurring lesson themes locally or account-backed if accounts exist later.
- Show trend summaries such as common mistake types and improving strengths.
- Let users compare today's strategy with prior games.

### Milestone 4: Considered Guesses

- Let users add optional considered guesses per turn.
- Compare them with the actual guess.
- Explain whether each alternative would have been better, worse, or differently useful.

## Resolved Product Decisions

- Post Game requires a solved board or a completed loss with six wrong guesses.
- Share text should include the official Wordle result line and grid plus a concise Wirdle coaching summary.
- Guess grades should be letter grades.
- Easy Mode should rely on escalation warnings rather than a per-game hint budget for now. The share text can make heavy hint usage visible.
- If the entered board is inconsistent with Wordle rules, Wirdle should not show Easy Mode hints or Post Game turn analysis until the board is fixed.

## Inconsistent Board State

Sometimes the user's entered guesses and letter statuses may be inconsistent with any possible Wordle answer.

Example: the user enters `CRANE` and marks `C` gray, then later enters `CLOUD` and marks `C` green in position 1. Those two pieces of feedback cannot both be true under Wordle rules, because a gray `C` in the first guess means the answer has no `C` at all.

This is a problem because Wirdle's coach depends on the set of possible answers that match the entered board. If the board is inconsistent, that set becomes empty. In Post Game, Wirdle cannot honestly grade whether a guess narrowed the answer space or whether an alternative would have been better. In Easy Mode, Wirdle cannot give reliable hints because there is no answer compatible with the board state.

Required behavior:

- Do not show Easy Mode hints.
- Do not show Post Game turn analysis.
- Do not show guess grades.
- Do not generate share text.
- Display a clear repair-focused error near the board.

Suggested message:

```text
This board does not match any possible Wordle answer. Check your tile colors and update the board before getting hints or analysis.
```

The message should be direct but not technical. It should frame the issue as a board-entry problem, not a user mistake.
