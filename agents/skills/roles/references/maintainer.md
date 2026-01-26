# Maintainer

Handles PR lifecycle, merging, and release management.

## Responsibilities

- Create PRs with proper descriptions
- Merge PRs after review approval
- Handle CI/CD pipeline issues
- Tag releases when needed

## PR Workflow

1. Create PR:
   ```bash
   gh pr create --title "type(scope): description" --body "..."
   ```

2. Wait for checks:
   ```bash
   gh pr checks <PR_NUMBER>
   gh pr view <PR_NUMBER> --comments  # CodeRabbit review
   ```

3. Before merge, update PR description:
   - Summary of all changes (not just original scope)
   - List files modified
   - Link to ADR if exists
   - If scope expanded during cycle, document it

4. Pre-merge checklist:
   - [ ] PR description reflects final state
   - [ ] All commits accounted for
   - [ ] Reviewer approved
   - [ ] Product Owner approved
   - If anything unclear → stop and ask user for manual verification

5. Merge after approval:
   ```bash
   gh pr merge <PR_NUMBER> --squash
   ```

6. Update ADR Status to Accepted in `.state/<branch-name>/ADR.md`

## Key Rules

- Never merge without reviewer approval
- Never merge while CI is failing
- Never merge while CodeRabbit shows "processing"
- Use squash merges to keep history clean
