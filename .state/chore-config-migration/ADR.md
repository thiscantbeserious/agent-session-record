# ADR: Config Migration with TOML-Edit Preservation

## Status
Accepted

## Context

When new config fields are added to the application (e.g., `filename_template`, `directory_max_length`), existing user config files don't contain these fields. While serde's `#[serde(default)]` handles this at runtime, users remain unaware of new options unless they read documentation.

Additionally, the current `src/config.rs` file (~248 lines) handles multiple responsibilities:
- Type definitions (Config, StorageConfig, ShellConfig, RecordingConfig, AgentsConfig)
- Default value functions (8 functions)
- Default trait implementations (4 impls)
- I/O operations (config_path, config_dir, load, save)
- Config helper methods (storage_directory, add_agent, remove_agent, etc.)

This makes it harder to add migration logic cleanly and test components in isolation.

**Forces at play:**
- Users customize their config files and add comments
- Config files should be self-documenting (show all available options)
- Existing customizations must never be lost
- The solution should feel professional and non-destructive
- Code organization should support future config-related features
- I/O operations should be testable in isolation

## Options Considered

### Migration Approach

#### Option 1: TOML Merge (Load + Default + Reserialize)
Load existing config, let serde defaults fill missing fields, serialize back.

- Pros: Simple, leverages existing serde infrastructure, defaults from `Config::default()`
- Cons: Loses comments and formatting, may reorder sections unexpectedly

#### Option 2: TOML-Edit Preservation (AST-level patching)
Use `toml_edit` crate to modify the document AST while preserving formatting.

- Pros: Preserves comments and formatting, minimal diff (only additions), professional UX
- Cons: Adds new dependency, more complex implementation

#### Option 3: Hybrid Show-Diff (Non-destructive display)
Don't modify the file; show what would change and let user manually update.

- Pros: Zero risk, simple implementation
- Cons: More friction for user, doesn't satisfy "adds missing fields" requirement directly

### Module Structure

#### Option A: Minimal Split (2 files)
- `mod.rs` (all existing code) + `migrate.rs` (new)
- Pros: Minimal change, fast to implement
- Cons: Doesn't address separation of concerns

#### Option B: Logical Split (3 files)
- `mod.rs` (re-exports + I/O + helpers), `types.rs`, `migrate.rs`
- Pros: Clear separation of "what" vs "how"
- Cons: I/O mixed with business logic

#### Option C: Full Separation (4 files)
- `mod.rs` (re-exports + helpers), `types.rs`, `io.rs`, `migrate.rs`
- Pros: Maximum separation, each file single responsibility, I/O testable
- Cons: More files to navigate

#### Option D: Domain Split (5+ files)
- Split by config section (storage.rs, recording.rs, agents.rs)
- Pros: Feature-organized
- Cons: Over-engineered for ~250 lines, unusual pattern

## Decision

**Migration: Option 2 - TOML-Edit Preservation**

We accept the trade-off of adding the `toml_edit` dependency in exchange for:
1. Preserving user comments and formatting
2. Producing minimal, reviewable diffs
3. Professional, non-destructive UX

The `toml_edit` crate is well-maintained and commonly used for this exact purpose.

**Module Structure: Option C - Full Separation**

```
src/config/
  mod.rs      - Re-exports + Config helper methods (~50 lines)
  types.rs    - Struct definitions + Default impls + default_* functions (~170 lines)
  io.rs       - config_path(), config_dir(), load(), save() (~45 lines)
  migrate.rs  - New migration logic using toml_edit (~100-150 lines)
```

This separation:
- Isolates the new `toml_edit` dependency to `migrate.rs` only
- Keeps type definitions clean and focused (no I/O deps)
- Makes I/O operations testable in isolation (mock filesystem)
- Prepares for future config features (validation.rs, schema.rs)
- Each file has a single, clear responsibility

## Consequences

**What becomes easier:**
- Users discover new config options without reading changelogs
- Config files stay self-documenting over time
- Upgrades feel seamless
- Testing I/O operations in isolation
- Adding new config-related features

**What becomes harder:**
- Must maintain parity between `Config` struct and TOML-edit logic
- New fields require consideration of where they appear in the document
- Navigating 4 files instead of 1 (mitigated by clear naming)

**Follow-ups to scope for later:**
- Schema versioning (if breaking changes ever needed)
- `--check` flag to verify config is up-to-date (CI use case)
- Config validation module

## Decision History

1. Command name decided as `agr config migrate` (per requirements)
2. User chose Option 2 (TOML-Edit Preservation) over simpler merge or display-only approaches
3. Safety approach: Preview with diff + confirm prompt (no backup file created)
4. Scope expanded: Refactor `src/config.rs` into module before adding migration logic
5. Module structure: Option C (Full Separation) with 4 files chosen after analyzing trade-offs
