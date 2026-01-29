# Requirements: Smart Filename Abbreviation

**Branch:** `chore/filename-truncation`
**Type:** Chore
**Sign-off:** Approved by user

## Problem Statement

The current `directory_max_length` truncation cuts at a fixed character limit, producing ugly results:
- `agent-session-recorder` at 15 chars → `agent-session-r` (partial word)
- `my-cool-project` at 10 chars → `my-cool-pr` (partial word + dangling separator implied)

## Desired Outcome

Smart abbreviation that shortens words proportionally to fit the limit while keeping all words recognizable:
- `agent-session-recorder` at 15 chars → `agnt-sess-rec` (all words preserved, abbreviated)
- `my-cool-project` at 10 chars → `my-col-prj` (all words preserved, abbreviated)

## Acceptance Criteria

- [ ] All words from input are represented in output (abbreviated if needed)
- [ ] No trailing separators in output
- [ ] Words shortened proportionally to fit limit
- [ ] Single words that exceed limit are hard truncated
- [ ] Short inputs that fit are unchanged
- [ ] Empty/whitespace input returns fallback value
- [ ] Existing tests updated, new edge case tests added

## Scope

**In scope:**
- Modify `sanitize_directory()` in `src/files/filename.rs`
- Update related tests

**Out of scope:**
- Changes to config schema
- Changes to other filename functions

## Decisions

- Word separators: `-`, `_`, `.`, and whitespace
- Abbreviation strategy: shorten each word proportionally
- Minimum word length: 2-3 chars to remain recognizable
- Separator preserved: use `-` as output separator
