# ADR: Improve Orchestrator SDLC Instructions

## Status
Accepted

## Context

The current orchestrator.md lacks clarity on several critical aspects:

1. **Structure issues**: The "Spawning Roles" section appears near the end (lines 105-119), but this is the primary mechanism for how the orchestrator operates. It should be prominent.

2. **Missing enforcement**: The document says "never implement code directly" but doesn't emphasize that the orchestrator should also never commit directly. Both are violations of the role boundary.

3. **SDLC scope unclear**: Users may assume the full SDLC workflow is only for "big features". In reality, ALL tasks benefit from the discipline: features, bugfixes, chores, small tasks. The consistency prevents shortcuts that lead to errors.

4. **Bypass exception undocumented**: The `/roles` command exists as a deliberate bypass for users who want direct role access. This exception should be documented within the boundaries section as the ONLY acceptable way to skip the full SDLC.

5. **Rules section weak**: The current "Rules" section is brief and buried. It needs expansion and better placement to emphasize boundaries.

6. **Orchestrator autonomy undefined**: The Orchestrator's role as a coordinator is implicit but not explicitly bounded. It should be clearly stated that the Orchestrator only relays messages and decisions between Agents - it must not form its own decisions or opinions about the work. Domain expertise belongs to specialized roles.

## Options Considered

### Option 1: Section Reordering with Expanded Rules

Restructure the document to:
1. Header & intro
2. **Spawning Roles** (moved up - HOW to use the system)
3. **Boundaries & Restrictions** (expanded from Rules, moved up)
4. **SDLC Scope** (NEW section - emphasizes ALL tasks)
5. **Roles** table
6. **Flow** diagram
7. **Steps**
8. **Responsibilities**
9. **State Files**
10. **Ambiguous Instructions**

- Pros: Emphasizes operational mechanics first; makes boundaries and scope unmissable; logical flow from "how" to "what" to "when"
- Cons: Requires careful restructuring; roles table moves down

### Option 2: Keep Structure, Add Inline Warnings

Keep the current structure but add prominent warnings/callouts throughout.

- Pros: Less invasive change; preserves familiarity
- Cons: Warnings can be ignored; doesn't fix the structural issue of important content being buried

### Option 3: Split into Multiple Files

Break orchestrator.md into separate files: spawning.md, boundaries.md, workflow.md.

- Pros: Each concern isolated; easier to maintain
- Cons: More files to manage; loses single-source-of-truth; harder to spawn orchestrator with full context

## Decision

**Option 1: Section Reordering with Expanded Rules**

This option addresses all identified issues:
- Places "Spawning Roles" immediately after intro (the HOW)
- Creates prominent "Boundaries & Restrictions" section (the MUST NOT)
- Adds explicit "SDLC Scope" section stating ALL tasks go through the cycle
- Documents `/roles` as the ONLY exception within Boundaries
- Maintains single-file simplicity for easy spawning
- Enforces Orchestrator as relay-only (no autonomous decisions or opinions)

Trade-offs accepted:
- Roles table moves down, but users already know roles exist
- Document gets slightly longer, but clarity is worth it

## Consequences

### What becomes easier
- New users understand the orchestrator's constraints immediately
- The "never skip SDLC" rule is explicit and unmissable
- The `/roles` bypass is documented as a deliberate escape hatch
- Spawning roles section is easy to find when needed

### What becomes harder
- Nothing significant; this is a documentation improvement

### Follow-ups to scope for later
- Consider adding examples of when `/roles` bypass is appropriate
- May need to update other role docs to reference the improved orchestrator

## Decision History

1. **Section reordering chosen** - Moving "Spawning Roles" and expanding "Rules" into "Boundaries & Restrictions" addresses the structural issues without fragmenting the document.

2. **SDLC applies to ALL tasks** - Explicitly stating that features, bugfixes, chores, and small tasks all go through the full cycle prevents users from taking shortcuts.

3. **`/roles` documented as rule breaker** - The bypass command is the ONLY acceptable exception to the full SDLC requirement, and it belongs in the Boundaries section as an explicit escape hatch.

4. **Orchestrator restrictions expanded** - "Never implement code" extended to include "never commit directly" to fully define the role boundary.

5. **Orchestrator as relay only** - The Orchestrator must only relay messages and decisions between Agents. It should not form its own decisions or opinions about the work being done. This ensures domain expertise stays with the specialized roles (Architect, Engineer, Reviewer) and the Orchestrator remains a neutral coordinator.
