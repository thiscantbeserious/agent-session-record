# ADR: Configurable Filename Template

## Status
Accepted

## Context

Current recording filenames use a hardcoded format (`%Y%m%d-%H%M%S-%3f.cast`) that is verbose and lacks project context. Users want:
1. Shorter, more readable datetime formats
2. Directory name in filename for project identification
3. Configurable templates for personal/team conventions

The existing `sanitize_filename()` in `recording.rs` is basic - it handles spaces and special chars but lacks:
- Length limits
- Unicode handling
- Reserved name detection (Windows `CON`, `NUL`, etc.)
- Proper test coverage for edge cases

Requirements mandate TDD approach for sanitization guards.

## Options Considered

### Option 1: Extend Existing (recording.rs)
Modify `generate_filename()` and `sanitize_filename()` in place.

- **Pros:** Fewer files, quick changes
- **Cons:** Mixes concerns, harder to test sanitization in isolation, `recording.rs` grows complex

### Option 2: New Dedicated Module (filename.rs)
Create `src/filename.rs` with clean separation of template parsing, sanitization, and generation.

- **Pros:**
  - Clean separation of concerns
  - Easy TDD - test sanitization as pure functions
  - Reusable for future features (rename, import)
  - Single responsibility
- **Cons:** New module (complexity warrants it)

## Decision

**Option 2: New `src/files/filename.rs` module**

Structure:
```
src/files/
├── mod.rs             # pub mod filename;
└── filename.rs
    ├── Template           # Parse and render templates
    │   ├── parse()        # Parse template string with tags
    │   ├── render()       # Render with current context (directory, datetime)
    │   └── DEFAULT        # Default template constant
    │
    ├── sanitize()         # All sanitization guards
    │   ├── replace spaces → hyphens
    │   ├── remove invalid chars (/\:*?"<>|)
    │   ├── handle unicode (transliterate/remove)
    │   ├── truncate directory to max length
    │   ├── trim leading/trailing dots/spaces
    │   └── handle reserved names
    │
    ├── generate()         # Main entry point using config
    │
    └── Config additions
        ├── filename_template: String
        └── directory_max_length: usize
```

The `src/files/` namespace allows future growth:
```
src/files/
├── mod.rs
├── filename.rs     # naming/templates (this feature)
├── paths.rs        # path utilities (later)
└── validation.rs   # file validation (later)
```

The module is a **shared utility** - decoupled from recording, usable by future rename/import commands.

### Template Syntax

```
{directory}           → current working directory name
{date}                → date with default format (%y%m%d)
{date:%Y-%m-%d}       → date with custom strftime format
{time}                → time with default format (%H%M)
{time:%H:%M:%S}       → time with custom strftime format
```

Default template: `{directory}_{date:%y%m%d}_{time:%H%M}`
Example output: `agent-session-recorder_260129_1714.cast`

### Sanitization Rules

1. Spaces → hyphens
2. Invalid filesystem chars (`/\:*?"<>|`) → removed
3. Unicode → ASCII transliteration where possible, else removed
4. `{directory}` truncated to `directory_max_length` (default: 50)
5. Leading/trailing dots and spaces → trimmed
6. Windows reserved names (`CON`, `PRN`, `NUL`, etc.) → prefixed with `_`
7. Empty result after sanitization → fallback to `recording`
8. Final filename > 255 chars → error

## Consequences

### What becomes easier
- Users can customize filename format via config
- Filenames include project context by default
- Future rename/import commands reuse sanitization
- Sanitization thoroughly tested via TDD

### What becomes harder
- Nothing significant

### Follow-ups to scope for later
- Additional template tags (`{agent}`, `{hostname}`, `{git_branch}`)
- Rename command using same sanitization
- Import command for external recordings

## Decision History

1. **Option 2 selected** - Dedicated module for clean separation and TDD-friendly testing.
2. **Module name: `filename`** - Clear and descriptive, user preference over `naming`.
3. **Format strings from start** - Include strftime support in v1 rather than deferring.
4. **Directory length configurable** - Only truncate `{directory}`, never date/time.
5. **Base module: `src/files/`** - Namespace for future file-related utilities (paths, validation).
