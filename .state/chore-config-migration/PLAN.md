# Plan: Config Migration Command

References: ADR.md

## Overview

Implement `agr config migrate` command that adds missing fields to user config files while preserving comments and formatting. Prerequisite: refactor `src/config.rs` into a proper module structure.

## Open Questions

Implementation challenges to solve (architect identifies, implementer resolves):

1. **Default document generation:** How to generate the reference default TOML for comparison? (Recommended: serialize `Config::default()` to string, parse with `toml_edit`)
2. **Nested table handling:** How to handle nested tables like `[storage]`, `[recording]` when merging? (Walk each section, add missing keys)
3. **Section ordering:** Where to insert new top-level sections if they don't exist? (Recommended: end of document, matching order in types.rs)
4. **Empty config edge case:** What if config file exists but is empty? (Treat as valid, add all sections)

## Stages

### Stage 1: Create config module structure

**Goal:** Set up the new `src/config/` directory structure without changing any logic.

**Tasks:**
- [x] Create `src/config/` directory
- [x] Create `src/config/types.rs` - move all struct definitions, `default_*` functions, and `Default` impls
- [x] Create `src/config/io.rs` - move `config_path()`, `config_dir()`, `load()`, `save()`
- [x] Create `src/config/mod.rs` - re-exports and remaining `Config` impl methods
- [x] Delete `src/config.rs`
- [x] Update `src/lib.rs` module declaration
- [x] Verify: `cargo build` succeeds
- [x] Verify: `cargo test` passes (no regressions)

**Files:**
- Delete: `src/config.rs`
- Create: `src/config/mod.rs`, `src/config/types.rs`, `src/config/io.rs`
- Modify: `src/lib.rs`

**Target file contents:**

`types.rs` (~170 lines):
- `Config`, `StorageConfig`, `AgentsConfig`, `ShellConfig`, `RecordingConfig` structs
- All `default_*` functions (8 total)
- All `Default` trait implementations (4 total)

`io.rs` (~45 lines):
- `pub fn config_path() -> Result<PathBuf>`
- `pub fn config_dir() -> Result<PathBuf>`
- `pub fn load() -> Result<Config>`
- `pub fn save(config: &Config) -> Result<()>`

`mod.rs` (~50 lines):
- `mod types;` / `mod io;` declarations
- `pub use types::*;` re-exports
- `Config` impl block with helper methods:
  - `storage_directory()`
  - `add_agent()`, `remove_agent()`, `is_agent_enabled()`
  - `should_wrap_agent()`, `add_no_wrap()`, `remove_no_wrap()`
- Wrapper methods that delegate to `io` module:
  - `Config::config_path()` -> `io::config_path()`
  - `Config::config_dir()` -> `io::config_dir()`
  - `Config::load()` -> `io::load()`
  - `Config::save(&self)` -> `io::save(self)`

**Verification:**
```bash
cargo build
cargo test
cargo clippy
```

---

### Stage 2: Add toml_edit dependency

**Goal:** Add the `toml_edit` crate for AST-preserving TOML manipulation.

**Tasks:**
- [x] Add `toml_edit = "0.22"` to `[dependencies]` in `Cargo.toml`
- [x] Verify: `cargo build` succeeds
- [x] Verify: no conflicts with existing `toml` crate

**Files:**
- Modify: `Cargo.toml`

**Notes:**
- `toml_edit` and `toml` can coexist (different purposes)
- Use latest stable version compatible with Rust 2021 edition

---

### Stage 3: Implement migration logic

**Goal:** Create the core migration function that merges default fields into existing config.

**Tasks:**
- [x] Create `src/config/migrate.rs`
- [x] Define `MigrateResult` struct: `{ content: String, added_fields: Vec<String>, sections_added: Vec<String> }`
- [x] Implement `pub fn migrate_config(existing_content: &str) -> Result<MigrateResult>`
- [x] Handle: completely empty input (return full default config)
- [x] Handle: partial config (add missing sections and fields)
- [x] Handle: complete config (return unchanged, empty added lists)
- [x] Preserve: existing values, comments, formatting, unknown fields
- [x] Add `mod migrate;` and `pub use migrate::*;` to `mod.rs`
- [x] Write unit tests covering all cases

**Files:**
- Create: `src/config/migrate.rs`
- Modify: `src/config/mod.rs`

**Algorithm outline:**
```rust
pub fn migrate_config(existing: &str) -> Result<MigrateResult> {
    // 1. Parse existing content with toml_edit
    let mut doc = existing.parse::<DocumentMut>()?;

    // 2. Generate default config as reference
    let default_toml = toml::to_string_pretty(&Config::default())?;
    let default_doc = default_toml.parse::<DocumentMut>()?;

    // 3. Walk default doc, add missing keys to user doc
    let mut added = Vec::new();
    for (section, table) in default_doc.iter() {
        // Add missing sections, then missing keys within sections
    }

    // 4. Return result
    Ok(MigrateResult {
        content: doc.to_string(),
        added_fields: added,
        sections_added: ...,
    })
}
```

**Unit tests:**
- Empty string input -> full default config
- Config with one section -> adds other sections
- Config with partial section -> adds missing fields
- Complete config -> no changes, empty added list
- Config with comments -> comments preserved
- Config with unknown fields -> unknown fields preserved

---

### Stage 4: Add CLI subcommand

**Goal:** Wire up `agr config migrate` in the CLI.

**Tasks:**
- [x] Add `Migrate` variant to `ConfigCommands` enum in `src/cli.rs`
- [x] Add short help and `long_about` documentation
- [x] Add `handle_migrate() -> Result<()>` in `src/commands/config.rs`
- [x] Add match arm in `main.rs` dispatch
- [x] Add CLI parsing test in `main.rs`

**Files:**
- Modify: `src/cli.rs`, `src/commands/config.rs`, `src/main.rs`

**CLI definition:**
```rust
// In ConfigCommands enum
/// Migrate config file to include all current fields
#[command(long_about = "Add missing fields to your config file.

Scans your config file and adds any fields that exist in the current
version but are missing from your file. Preserves your existing values,
comments, and formatting.

Shows a preview of changes and asks for confirmation before writing.

EXAMPLE:
    agr config migrate")]
Migrate,
```

**Handler (initial - just calls migrate and prints):**
```rust
pub fn handle_migrate() -> Result<()> {
    let config_path = Config::config_path()?;
    let content = std::fs::read_to_string(&config_path)
        .unwrap_or_default();

    let result = agr::config::migrate_config(&content)?;

    if result.added_fields.is_empty() {
        println!("Config is up to date.");
    } else {
        println!("Would add {} fields", result.added_fields.len());
        // Preview logic added in Stage 5
    }
    Ok(())
}
```

---

### Stage 5: Implement preview and confirmation flow

**Goal:** Show colored diff and prompt user before writing changes.

**Tasks:**
- [x] If no changes needed: print "Config is already up to date" and exit
- [x] If changes needed:
  - [x] Print summary: "Found N missing fields in M sections"
  - [x] Show colored diff (green `+` for additions)
  - [x] List added fields/sections
  - [x] Prompt: "Apply these changes? [y/N]"
- [x] On confirm (`y` or `yes`): write new content to config file
- [x] On decline (anything else): exit without changes
- [x] Handle: config file doesn't exist -> create with full defaults (with confirmation)

**Files:**
- Modify: `src/commands/config.rs`

**UX flow:**
```
$ agr config migrate

Found 3 missing fields in 1 section:

  [recording]
+ filename_template = "{directory}_{date}_{time}"
+ directory_max_length = 50
+ auto_analyze = false

Apply these changes to ~/.config/agr/config.toml? [y/N] y

Config updated successfully.
```

**Considerations:**
- Use `crossterm` or similar for colors (check what's already in deps)
- Check `atty::is(Stream::Stdout)` for color support
- Atomic write: write to temp file, then rename (for safety)

---

### Stage 6: Tests

**Goal:** Comprehensive test coverage for the new feature.

**Tasks:**
- [x] Unit tests for `migrate_config()` (in `migrate.rs`) - covered in Stage 3 (10 tests)
- [x] CLI parsing test: `agr config migrate` parses correctly - added in Stage 4
- [x] Integration test: migrate on file with missing fields - covered by unit test `partial_section_adds_missing_fields`
- [x] Integration test: migrate on up-to-date file reports no changes - covered by unit test `complete_config_returns_no_changes`
- [x] Integration test: migrate preserves comments - covered by unit test `comments_are_preserved`
- [x] Integration test: decline prompt leaves file unchanged - N/A (prompt is in handler, not migrate_config)

**Files:**
- Modify: `src/main.rs` (CLI tests)
- Modify: `src/config/migrate.rs` (unit tests)
- Optionally: `tests/config_migrate.rs` (integration tests)

**Test fixtures needed:**
- Partial config TOML (missing some fields)
- Complete config TOML (all fields present)
- Config with comments TOML
- Config with unknown fields TOML

---

## Dependencies

```
Stage 1 (refactor module)
    │
    ▼
Stage 2 (add toml_edit) ──► Stage 3 (migrate logic)
                                    │
                                    ▼
                              Stage 4 (CLI) ──► Stage 5 (UX) ──► Stage 6 (tests)
```

- Stage 1 must complete first (establishes module structure)
- Stage 2 can start immediately after Stage 1
- Stage 3 requires Stage 2 (needs `toml_edit` crate)
- Stage 4 requires Stage 3 (needs `migrate_config` function)
- Stage 5 requires Stage 4 (needs CLI wiring)
- Stage 6 can run after Stage 5 (tests complete feature)

## Progress

Updated by implementer as work progresses.

| Stage | Status | Notes |
|-------|--------|-------|
| 1 | done | Refactored config.rs into module (types.rs, io.rs, mod.rs) |
| 2 | done | Added toml_edit = "0.22" to Cargo.toml |
| 3 | done | Implemented migrate_config() with MigrateResult, 10 unit tests |
| 4 | done | Added CLI subcommand `agr config migrate` with handler and test |
| 5 | done | Preview diff, confirmation prompt, TTY detection, create/update flow |
| 6 | done | All tests verified: 10 unit tests + 1 CLI parsing test |
