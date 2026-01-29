# Requirements: Configurable Filename Template

## Problem Statement

Current recording filenames use a fixed format (`20260129-171400-123.cast`) that is:
1. Verbose and hard to read at a glance
2. Missing context about where the recording was made
3. Not customizable to user preferences

Users want filenames that:
- Include the directory name where the agent was invoked (project context)
- Use a shorter, more readable datetime format
- Can be configured to match personal or team conventions

## User Stories

- As a user, I want my recording filenames to include the project directory so I can identify recordings by project without opening them
- As a user, I want shorter datetime formats (YYMMDD vs YYYYMMDD) so filenames are more scannable
- As a user, I want to configure the filename format in my config file so I can customize it to my preferences

## Desired Outcome

A template-based naming system where users can define their own filename pattern using template tags, with a sensible default that improves on the current format.

## Acceptance Criteria

### Template System
- [ ] New `filename_template` config option in `[recording]` section
- [ ] Default template: `{directory}_{date:%y%m%d}_{time:%H%M}` producing e.g., `agent-session-recorder_260129_1714.cast`
- [ ] Supported template tags:
  - `{directory}` - name of the current working directory
  - `{date}` or `{date:FORMAT}` - date with optional strftime format (default: `%y%m%d`)
  - `{time}` or `{time:FORMAT}` - time with optional strftime format (default: `%H%M`)
- [ ] Format strings use chrono's strftime syntax (e.g., `%Y-%m-%d`, `%H:%M:%S`)
- [ ] Template is applied when auto-generating filenames (not when user provides explicit name via `-n`)
- [ ] Invalid templates or format strings produce clear error messages

### Filename Sanitization (Test-First)
- [ ] Spaces replaced with hyphens or underscores
- [ ] Invalid filesystem characters removed/replaced (e.g., `/`, `\`, `:`, `*`, `?`, `"`, `<`, `>`, `|`)
- [ ] Unicode/special characters handled gracefully (transliterate or remove)
- [ ] Empty results after sanitization produce fallback name
- [ ] Leading/trailing dots and spaces trimmed
- [ ] Reserved names handled (e.g., Windows `CON`, `PRN`, `NUL`)
- [ ] Tests written BEFORE implementation (TDD approach)

### Length Limits
- [ ] New `directory_max_length` config option (default: 50 chars)
- [ ] Only `{directory}` tag is truncated, never date/time
- [ ] Truncation adds no ellipsis (clean cut for filesystem compatibility)
- [ ] Total filename still validated against 255 char filesystem limit (error if exceeded after all processing)

### Documentation
- [ ] README updated to explain the template system, available tags, and format string syntax

## Out of Scope

- Additional tags beyond directory/date/time (can add more later)
- Changing existing behavior of `-n` flag for explicit naming
- Migration of existing recordings to new naming format

## Sign-off

- [x] User approved
