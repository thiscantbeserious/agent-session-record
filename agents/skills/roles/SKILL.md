---
name: roles
description: Agent role definitions. Load when assigned a role and read the matching file from references/ (only the role you are supposed to take). If asked for a list return a numbered list of all roles and give the user the option to choose by number or name.
---

# Agent Roles

## 1. Access pattern

Read the role file from `references/` that matches your assignment (default: orchestrator). After loading your role, check instructions for task-specific guidance that are either directly mentioned or make sense for the domain dynamically as you go.

## 2. Restriction

Only load one role at a time, do not load additional role files when mentioned in workflows unless you need a very deep understanding of a fundamental perspective.
