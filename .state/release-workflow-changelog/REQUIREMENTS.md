# Requirements: Release Workflow & Changelog

## Problem Statement

The project lacks a formal release workflow. There are:
- No git tags for versioning
- No CHANGELOG.md to communicate changes to users
- No automated release process triggered by tags
- Stale state directories accumulating in `.state/`
- INDEX.md learnings not being captured at end of SDLC cycles
- decisions.md is stale (last updated at PR #24, now at PR #74)

Users and contributors have no way to track releases, understand what changed between versions, or consume stable releases.

## Context

- Project is at ~74 merged PRs with no releases tagged
- CI currently builds artifacts on every main push (wasteful)
- git-cliff will be used for changelog generation (external tool)
- Maintainer manually decides when to release (batched, not per-PR)
- Releases trigger on git tag push, not main push

## Requirements

### 1. Git Tags & Versioning

1.1. Use semantic versioning starting at v0.1.0
1.2. Version scheme:
   - `0.x.0` -> `0.(x+1).0` for feature releases
   - `0.x.y` -> `0.x.(y+1)` for bugfix-only releases
1.3. Tags are created manually by the Maintainer
1.4. Tag format: `v{major}.{minor}.{patch}` (e.g., `v0.1.0`, `v0.2.0`, `v0.2.1`)

### 2. Changelog Generation (git-cliff)

2.1. Install and configure git-cliff with `cliff.toml`
2.2. Changelog categories based on conventional commits:
   - **Features** - `feat:` commits
   - **Bug Fixes** - `fix:` commits
   - **Performance** - `perf:` commits
   - **Refactor** - `refactor:` commits
   - **Documentation** - `docs:` commits
   - **Chore** - `chore:` commits
   - **Style** - `style:` commits
2.3. Generate CHANGELOG.md from PR titles/commit messages
2.4. Maintainer may optionally refine generated changelog before release
2.5. Changelog should link to PRs/commits where applicable

### 3. CI/CD Release Workflow

3.1. Create new workflow file: `.github/workflows/release.yml`
3.2. Trigger on tag push matching `v*` pattern
3.3. Workflow steps:
   - Run all existing checks (build, test, lint, etc.)
   - Generate changelog for the release (or verify it exists)
   - Build release artifacts for all platforms (linux-x86_64, macos-x86_64, macos-arm64)
   - Create GitHub Release with:
     - Tag name as release title
     - Changelog section for this version as release notes
     - Attached binary artifacts
3.4. Remove artifact creation from main push (move to tag-only)
3.5. Keep PR/main checks as-is (build, test, lint, coverage)

### 4. Maintainer Role Updates

4.1. Add end-of-cycle tasks to Maintainer role:
   - Update INDEX.md "Recently completed" section
   - Capture learnings in decisions.md (if any significant learnings)
   - Ensure state directory has complete ADR/PLAN/REQUIREMENTS for future LLM reference
4.2. Add release process documentation:
   - When to release (discretionary)
   - How to create a release (tag + push)
   - How to generate/refine changelog

### 5. INDEX.md Learnings Capture

5.1. Update INDEX.md structure to include learnings section or keep current "Recently completed" format
5.2. At end of each SDLC cycle, Maintainer adds entry to "Recently completed"
5.3. Format: Brief description with PR number reference
5.4. Keep list trimmed to ~5-10 most recent items (archive older)

### 6. State Directory as LLM Context (INDEX.md as catalog)

6.1. **KEEP all `.state/<branch-name>/` directories** - they are valuable context for future LLM sessions
6.2. INDEX.md becomes the **entry point for LLMs** with a "Completed Work" section that:
   - Lists ALL state directories
   - Includes one sentence describing the purpose of each
   - Links to the directory for full ADR/PLAN/REQUIREMENTS
6.3. **Immediate action:** Populate INDEX.md with all 12 existing state directories + descriptions
6.4. **Future process:** Maintainer adds entry to INDEX.md "Completed Work" at end of each SDLC cycle
6.5. **Description format:** Short (5-10 words), problem/area focused, helps LLM decide relevance
   - GOOD: "Silence/pause removal for recordings"
   - GOOD: "Terminal scroll region support in player"
   - BAD: "Implemented a two-phase review process where the Reviewer role is spawned twice..."
   - Rule: Describe WHAT area/problem, not HOW it was implemented
6.6. No archival or deletion - directories persist indefinitely as knowledge base

## Acceptance Criteria

- [ ] `cliff.toml` exists and is properly configured for conventional commits
- [ ] Running `git-cliff` generates valid CHANGELOG.md
- [ ] `.github/workflows/release.yml` exists and triggers on `v*` tags
- [ ] Tag push creates GitHub Release with artifacts and changelog notes
- [ ] Main push no longer creates release artifacts (only runs checks)
- [ ] INDEX.md has "Completed Work" section listing all state directories with descriptions
- [ ] All 12 existing state directories have entries in INDEX.md with one-sentence purpose
- [ ] All existing state directories are preserved (not archived/deleted)
- [ ] Maintainer role documentation includes end-of-cycle tasks
- [ ] Maintainer role documentation includes release process
- [ ] INDEX.md "Recently completed" is updated with recent work
- [ ] First release v0.1.0 can be successfully created

## Out of Scope

- Automated version bumping (versions are manual)
- Release branches (releases are from main via tags)
- Pre-release versions (alpha, beta, rc)
- Automated changelog refinement (Maintainer does this manually if needed)
- Crates.io publishing (not requested)
- Docker image releases (not requested)
- decisions.md full refresh (only add process for future learnings)
- State directory archival/deletion (directories are kept as LLM context)

## Open Questions

None - all questions resolved during requirements interview.

## Technical Notes

### git-cliff Configuration

The `cliff.toml` should:
- Parse conventional commit prefixes
- Group by category
- Include PR links via GitHub integration
- Support unreleased changes section

### CI Workflow Structure

```
Current ci.yml:
  - on: push main, PR main
  - jobs: build, unit-tests, coverage, e2e-tests, snapshot-tests, lint
  - release job: on main push only (TO BE REMOVED)

New release.yml:
  - on: push tags v*
  - jobs: all checks + create GitHub Release with artifacts
```

### Maintainer End-of-Cycle Checklist (New)

1. Merge PR
2. Update INDEX.md "Recently completed"
3. Add to decisions.md if significant learnings
4. Ensure `.state/<branch-name>/` directory has complete ADR, PLAN, REQUIREMENTS for future reference
5. (Optional) Create release tag if batching complete
