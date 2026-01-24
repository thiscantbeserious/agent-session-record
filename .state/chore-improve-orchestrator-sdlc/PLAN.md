# Plan: Improve Orchestrator SDLC Instructions

References: ADR.md

## Open Questions

1. Should the "Ambiguous Instructions" section be expanded with more examples?
2. Should we add a version/changelog header to track document evolution?

## Stages

### Stage 1: Restructure orchestrator.md

Goal: Reorder sections to place operational mechanics first and expand boundaries

- [ ] Move "Spawning Roles" section to position 2 (after header/intro)
- [ ] Rename "Rules" to "Boundaries & Restrictions" and expand content
- [ ] Move expanded "Boundaries & Restrictions" to position 3
- [ ] Add new "SDLC Scope" section at position 4
- [ ] Reorder remaining sections: Roles, Flow, Steps, Responsibilities, State Files, Ambiguous Instructions

Files: `.claude/skills/roles/references/orchestrator.md`

Considerations:
- Preserve all existing content; this is restructuring, not removal
- Ensure Mermaid/ASCII diagram remains intact
- Keep markdown formatting consistent

### Stage 2: Expand Boundaries & Restrictions

Goal: Transform the brief "Rules" into comprehensive boundaries

- [ ] Add "never commit directly" alongside "never implement code"
- [ ] Add "relay only" restriction: Orchestrator must not form own decisions or opinions
- [ ] Document that Orchestrator only passes messages/decisions between Agents
- [ ] Document `/roles` command as the ONLY exception to full SDLC
- [ ] Add clear warning that bypassing SDLC without `/roles` violates protocol
- [ ] Number the restrictions for easy reference

Files: `.claude/skills/roles/references/orchestrator.md`

Considerations:
- Keep language direct and unambiguous
- The `/roles` exception should be framed as "deliberate escape hatch" not "recommended shortcut"
- The relay-only restriction ensures domain expertise stays with specialized roles (Architect, Engineer, Reviewer)

### Stage 3: Add SDLC Scope Section

Goal: Explicitly state that ALL tasks go through the full SDLC

- [ ] Create new section between Boundaries and Roles table
- [ ] List task types: features, bugfixes, chores, refactoring, documentation
- [ ] Emphasize consistency prevents shortcuts that lead to errors
- [ ] Reference that even "small" tasks benefit from the discipline

Files: `.claude/skills/roles/references/orchestrator.md`

Considerations:
- Keep section concise but emphatic
- Avoid making it feel like bureaucratic overhead; frame as quality assurance

### Stage 4: Final Review and Validation

Goal: Ensure the restructured document is complete and consistent

- [ ] Verify all original content preserved
- [ ] Check section numbering and headings
- [ ] Validate internal references (if any)
- [ ] Confirm markdown renders correctly

Files: `.claude/skills/roles/references/orchestrator.md`

Considerations:
- Read through as a new user would
- Ensure the document still fits in a single spawn prompt

## Dependencies

- Stage 2 depends on Stage 1 completing the restructure (section must exist to expand)
- Stage 3 depends on Stage 1 (new section needs the structure in place)
- Stage 4 depends on all previous stages completing

## Progress

Updated by implementer as work progresses.

| Stage | Status | Notes |
|-------|--------|-------|
| 1 | completed | Restructured sections: Spawning Roles to pos 2, Boundaries to pos 3, SDLC Scope to pos 4 |
| 2 | completed | Added restrictions #2 (never commit), #3 (relay only), /roles exception documented |
| 3 | completed | Added SDLC Scope section with task types and consistency emphasis |
| 4 | completed | All original content preserved, diagram intact, markdown validated |
