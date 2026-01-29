# Implementation Plan: First Syllable Heuristic Abbreviation

**Branch:** `chore/filename-truncation`
**Scope:** `src/files/filename.rs`

## Algorithm: First Syllable Heuristic

### Overview
When a directory name exceeds the configured max length, abbreviate each word to its first syllable rather than hard-truncating.

### Steps
1. If input fits within limit, return unchanged
2. Split on word separators (`-`, `_`, `.`, whitespace)
3. For each word, extract first syllable:
   - Find the first vowel (a, e, i, o, u)
   - Include consonants after the first vowel until the next vowel or end of word
   - Examples:
     - `session` → `ses` (s + e + s, stop before 'i')
     - `recorder` → `rec` (r + e + c, stop before 'o')
     - `project` → `proj` (p + r + o + j, stop before 'e')
     - `agent` → `ag` (a + g, stop before 'e')
     - `hello` → `hel` (h + e + l, stop before second 'l' which precedes 'o')
     - `world` → `world` (only one vowel, keep all)
     - `awesome` → `aw` (a + w, stop before 'e')
     - `testing` → `test` (t + e + s + t, stop before 'i')
     - `example` → `ex` (e + x, stop before 'a')
4. If first-syllable result still exceeds limit, truncate proportionally
5. Join abbreviated words with `-`
6. Ensure no leading/trailing hyphens, no double hyphens

### First Syllable Examples

| Word | First Syllable | Explanation |
|------|----------------|-------------|
| `agent` | `ag` | a(vowel) + g(consonant), stop at 'e' |
| `session` | `ses` | s + e(vowel) + s, stop at 'i' |
| `recorder` | `rec` | r + e(vowel) + c, stop at 'o' |
| `project` | `proj` | p + r + o(vowel) + j, stop at 'e' |
| `hello` | `hel` | h + e(vowel) + l, stop at 'l' before 'o' |
| `world` | `world` | w + o(vowel) + r + l + d, only one vowel |
| `awesome` | `aw` | a(vowel) + w, stop at 'e' |
| `testing` | `test` | t + e(vowel) + s + t, stop at 'i' |
| `example` | `ex` | e(vowel) + x, stop at 'a' |
| `my` | `my` | short word, keep as-is |
| `cool` | `co` | c + o(vowel), stop at second 'o' (a vowel) |
| `three` | `three` | all consonants then vowels at end, keep all |
| `one` | `one` | short word, keep as-is |
| `two` | `two` | short word, keep as-is |
| `four` | `fo` | f + o(vowel), stop at 'u' (a vowel) |
| `five` | `fiv` | f + i(vowel) + v, stop at 'e' |
| `seven` | `sev` | s + e(vowel) + v, stop at second 'e' |
| `eight` | `e` | e(vowel), stop immediately at 'i' |

### Full Input Examples

| Input | First Syllable Result | Length |
|-------|----------------------|--------|
| `agent-session-recorder` | `ag-ses-rec` | 10 |
| `hello-world` | `hel-world` | 9 |
| `my-cool-project` | `my-co-proj` | 10 |
| `one-two-three-four` | `one-two-three-four` | 18 (unchanged, fits limit) |
| `my-awesome-project` | `my-aw-proj` | 10 |
| `testing-example` | `test-ex` | 7 |
| `my-really-awesome-cool-project` | `my-re-aw-co-proj` | 16 |

## Changes

### 1. Add helper function `first_syllable()`

```rust
/// Extracts the first syllable of a word.
/// Returns the word up to and including consonants after the first vowel.
fn first_syllable(word: &str) -> &str {
    const VOWELS: &[char] = &['a', 'e', 'i', 'o', 'u', 'A', 'E', 'I', 'O', 'U'];

    let chars: Vec<char> = word.chars().collect();

    // Find first vowel
    let first_vowel_idx = chars.iter().position(|c| VOWELS.contains(c));
    let Some(vowel_idx) = first_vowel_idx else {
        return word; // No vowel, return whole word
    };

    // Find next vowel after first vowel
    let after_vowel = &chars[vowel_idx + 1..];
    let next_vowel_offset = after_vowel.iter().position(|c| VOWELS.contains(c));

    match next_vowel_offset {
        Some(offset) => {
            // Return up to (but not including) the next vowel
            let end_idx = vowel_idx + 1 + offset;
            &word[..word.char_indices().nth(end_idx).map(|(i, _)| i).unwrap_or(word.len())]
        }
        None => word, // Only one vowel, return whole word
    }
}
```

### 2. Replace `truncate_to_length()` function (lines 198-204)

**Current:**
```rust
fn truncate_to_length(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        s.chars().take(max_len).collect()
    }
}
```

**New:**
```rust
fn truncate_to_length(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }

    // Split on word boundaries
    let words: Vec<&str> = s
        .split(|c| c == '-' || c == '_' || c == '.')
        .filter(|w| !w.is_empty())
        .collect();

    // Single word: just truncate
    if words.len() <= 1 {
        return s.chars().take(max_len).collect();
    }

    // Apply first syllable to each word
    let abbreviated: Vec<&str> = words.iter().map(|w| first_syllable(w)).collect();
    let result = abbreviated.join("-");

    // If still too long, truncate proportionally
    if result.len() <= max_len {
        return result;
    }

    // Further truncation needed - distribute chars evenly
    let separator_count = words.len() - 1;
    let available = max_len.saturating_sub(separator_count);
    let chars_per_word = available / words.len();

    abbreviated
        .iter()
        .map(|w| w.chars().take(chars_per_word.max(1)).collect::<String>())
        .collect::<Vec<_>>()
        .join("-")
}
```

## Files Modified

- `src/files/filename.rs` - truncation logic + helper function
- `tests/integration/filename_test.rs` - comprehensive test suite

## Verification

```bash
cargo test filename
cargo clippy
```
