# ADR: Smart Abbreviation for Directory Names

**Status:** Accepted
**Date:** 2025-01-29
**Branch:** `chore/filename-truncation`

## Context

The `truncate_to_length()` function in `src/files/filename.rs` currently performs hard character truncation, producing suboptimal results:

| Input | Limit | Current | Desired |
|-------|-------|---------|---------|
| `agent-session-recorder` | 15 | `agent-session-r` | `agnt-sess-rec` |
| `my-awesome-project` | 12 | `my-awesome-p` | `my-awsm-proj` |
| `very-long-directory-name` | 10 | `very-long-` | `vry-lng-dir` |

Hard truncation leaves partial words that are harder to read than abbreviated words.

## Decision

Replace hard truncation with **smart abbreviation** that distributes available characters across all words proportionally.

### Algorithm

1. **Split** input on separators (`-`, `_`, `.`, whitespace)
2. **Calculate** chars per word: `(limit - separator_count) / word_count`
3. **Truncate** each word to calculated length (minimum 3 chars for readability)
4. **Join** with hyphen separator

### Edge Cases

| Case | Behavior |
|------|----------|
| Single word | Truncate to limit |
| Input already fits | Return unchanged |
| Too many words | Each word gets minimum (3 chars), may slightly exceed limit |
| Empty input | Handled by existing `sanitize()` fallback |

## Consequences

**Positive:**
- Abbreviated filenames remain readable and recognizable
- All words from original name are represented
- Consistent output format with hyphen separators

**Negative:**
- Output may slightly exceed limit when many words require minimum length
- Original separator style not preserved (all become hyphens)

## Implementation

Single function change in `src/files/filename.rs`: replace `truncate_to_length()`.
