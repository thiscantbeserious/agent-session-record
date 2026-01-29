# Plan: Configurable Filename Template

References: ADR.md

## Open Questions

Implementation challenges to solve (architect identifies, implementer resolves):

1. Unicode transliteration - use `deunicode` crate or manual mapping? Check if already in dependency tree.
2. Should `{date}` and `{time}` be separate tags or combined into `{datetime}`? ADR specifies separate - verify this makes sense during implementation.

## Stages

### Stage 1: Sanitization Tests (TDD)

Goal: Write comprehensive tests for sanitization BEFORE implementation

- [ ] Create `src/filename.rs` with `sanitize()` function stub returning `todo!()`
- [ ] Write tests for space replacement → hyphens
- [ ] Write tests for invalid char removal (`/\:*?"<>|`)
- [ ] Write tests for unicode handling (é→e, 日本→removed or transliterated)
- [ ] Write tests for length truncation (directory only)
- [ ] Write tests for leading/trailing dot/space trimming
- [ ] Write tests for Windows reserved names (CON, PRN, NUL, etc.)
- [ ] Write tests for empty result fallback
- [ ] Write tests for final length validation (>255 chars)
- [ ] Verify all tests FAIL (red phase)

Files: `src/files/mod.rs`, `src/files/filename.rs`, `tests/integration/filename_test.rs`

Considerations:
- Tests should cover edge cases: empty string, only spaces, only dots, mixed unicode
- Test both individual sanitization steps and combined behavior

### Stage 2: Sanitization Implementation

Goal: Implement sanitization to make all tests pass

- [ ] Implement space → hyphen replacement
- [ ] Implement invalid char removal
- [ ] Implement unicode handling (decide on crate vs manual)
- [ ] Implement directory length truncation
- [ ] Implement leading/trailing trimming
- [ ] Implement reserved name handling
- [ ] Implement empty fallback
- [ ] Implement final length check
- [ ] Verify all tests PASS (green phase)
- [ ] Refactor if needed (refactor phase)

Files: `src/files/filename.rs`

Considerations:
- Order of operations matters: sanitize chars before truncating length
- Unicode transliteration should happen before other sanitization

### Stage 3: Template Parser Tests (TDD)

Goal: Write tests for template parsing BEFORE implementation

- [ ] Write tests for parsing `{directory}` tag
- [ ] Write tests for parsing `{date}` with default format
- [ ] Write tests for parsing `{date:%Y-%m-%d}` with custom format
- [ ] Write tests for parsing `{time}` with default format
- [ ] Write tests for parsing `{time:%H%M%S}` with custom format
- [ ] Write tests for literal text between tags
- [ ] Write tests for invalid/malformed templates (error cases)
- [ ] Write tests for empty template
- [ ] Verify all tests FAIL

Files: `src/files/mod.rs`, `src/files/filename.rs`, `tests/integration/filename_test.rs`

Considerations:
- Template parsing should be separate from rendering
- Invalid format strings should produce clear errors

### Stage 4: Template Parser Implementation

Goal: Implement template parsing to make tests pass

- [ ] Define `Template` struct with parsed segments
- [ ] Implement `Template::parse()`
- [ ] Handle `{directory}`, `{date}`, `{date:FORMAT}`, `{time}`, `{time:FORMAT}`
- [ ] Handle literal text segments
- [ ] Implement error handling for malformed templates
- [ ] Define `DEFAULT` constant template
- [ ] Verify all tests PASS

Files: `src/files/filename.rs`

Considerations:
- Use enum for segment types: Literal, Directory, Date(format), Time(format)
- Validate strftime format strings during parse, not render

### Stage 5: Template Rendering & Generation

Goal: Implement full filename generation

- [ ] Implement `Template::render()` with context (directory, datetime)
- [ ] Implement `generate()` main entry point
- [ ] Wire sanitization into render pipeline (sanitize directory before inserting)
- [ ] Add integration test for full generate flow
- [ ] Test with various directory names and templates

Files: `src/files/filename.rs`

Considerations:
- Sanitize `{directory}` value, not the template itself
- Date/time come from `chrono::Local::now()` - consider making injectable for testing

### Stage 6: Config Integration

Goal: Add config options and wire into recording

- [ ] Add `filename_template` to `RecordingConfig` with default
- [ ] Add `directory_max_length` to `RecordingConfig` with default (50)
- [ ] Update `Recorder::generate_filename()` to use new module
- [ ] Deprecate or remove old implementation
- [ ] Update existing recording tests

Files: `src/config.rs`, `src/recording.rs`, `src/lib.rs`

Considerations:
- Maintain backward compatibility during transition
- Config validation should catch invalid templates early

### Stage 7: Documentation & Cleanup

Goal: Update docs and finalize

- [ ] Update README with template documentation
- [ ] Document available tags and strftime syntax
- [ ] Add example configurations
- [ ] Run full test suite
- [ ] Run clippy and fix warnings

Files: `README.md`, `docs/`

Considerations:
- Include common template examples users might want
- Link to strftime reference for format strings

## Dependencies

```
Stage 1 (sanitize tests)
    → Stage 2 (sanitize impl)
        → Stage 3 (template tests)
            → Stage 4 (template impl)
                → Stage 5 (rendering)
                    → Stage 6 (config)
                        → Stage 7 (docs)
```

All stages are sequential - each builds on the previous.

## Progress

Updated by implementer as work progresses.

| Stage | Status | Notes |
|-------|--------|-------|
| 1 | complete | 53 tests written, 52 failing (RED phase) |
| 2 | complete | All 53 sanitization tests passing (GREEN phase) |
| 3 | complete | 25 template tests written, all failing (RED phase) |
| 4 | complete | Template parser + renderer implemented, all 78 tests pass |
| 5 | complete | generate() function added, all 84 tests pass |
| 6 | complete | Config integration done, recording.rs updated |
| 7 | complete | README updated with template documentation |
