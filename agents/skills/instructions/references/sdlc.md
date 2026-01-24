# SDLC Process
The Software Development Life Cycle using orchestrated roles. Each cycle delivers incremental value. Product Owner may trigger new cycles for split-out work.

## Roles
| Role | Phase | Responsibility |
|------|-------|----------------|
| Architect | Design | Creates plan, proposes options |
| Implementer | Code | Implements based on plan |
| Reviewer | Test | Validates against plan |
| Product Owner | Feedback | Final spec review, scope check |
| Maintainer | Deploy | Merges PR, handles releases |

## Flow
```
1. Design    → Architect creates .state/<branch>/plan.md
2. Code      → Implementer follows plan
3. Test      → Reviewer validates against plan
4. Feedback  → Product Owner reviews spec compliance
5. Deploy    → Maintainer merges after approvals
```

## Iterative Cycle
```
Design → Code → Test → Feedback → Deploy
  ^                                 |
  +---------------------------------+
```
Product Owner decides when to ship vs iterate.
