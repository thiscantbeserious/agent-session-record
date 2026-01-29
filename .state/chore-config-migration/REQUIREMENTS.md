# Requirements: Config Migration

**Branch:** `chore/config-migration`
**Type:** Chore
**Sign-off:** Approved by user

## Problem Statement

When new config fields are added (e.g., `filename_template`, `directory_max_length`), existing user config files don't contain these fields. While serde's `#[serde(default)]` handles this at runtime (missing fields get defaults), the config file itself remains outdated, making it unclear to users what options are available.

## Current Behavior

1. User has existing `~/.config/agr/config.toml` without new fields
2. App loads config - serde fills in defaults (works fine)
3. User runs `agr config show` - sees their config, but new fields are hidden
4. User doesn't know new options exist unless they read docs

## Desired Outcome

Users can easily update their config file to include all current fields with sensible defaults, making new options discoverable.

## Acceptance Criteria

- [ ] Command to refresh/migrate config file to include all current fields
- [ ] Preserves existing user values (doesn't overwrite customizations)
- [ ] Adds missing fields with their defaults
- [ ] Shows diff or summary of what changed
- [ ] Non-destructive: creates backup or requires confirmation

## Scope

**In scope:**
- Command to add missing fields to config file
- Preserve existing values
- Show what was added

**Out of scope:**
- Schema versioning system
- Breaking change migrations (field renames, type changes)
- Automatic migration on startup

## Decisions

1. **Command name:** `agr config migrate`
2. **Auto-run:** No - manual invocation only
3. **Safety:** Preview with diff + confirm (no backup file)
