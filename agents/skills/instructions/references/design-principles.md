# Design Principles

Architectural guidelines for planning and evaluating design decisions.

## Simplicity First

**Default to the simplest solution that works.**

Before adding abstraction, ask:
- Can this be a function instead of a trait?
- Can this be a module instead of a crate?
- Can this be inline instead of a separate file?
- Will someone understand this in 6 months?

Complexity must justify itself with concrete benefits, not hypothetical future needs.

## Module Boundaries

**Create a new module when:**
- A distinct responsibility emerges (single clear purpose)
- The code would exceed 400 lines in current location
- Multiple files would benefit from shared private helpers
- Testing requires isolation from other components

**Keep code in existing module when:**
- It's closely related to existing functionality
- It would create a module with only 1-2 small functions
- The "new module" would just re-export or wrap existing code

## Abstraction Decisions

**Abstraction is justified when:**
- There are 3+ concrete implementations (not 2, not "maybe later")
- The abstraction simplifies the calling code
- Testing becomes easier with the abstraction

**Avoid abstraction when:**
- "We might need this flexibility later"
- There's only one implementation
- The abstraction adds indirection without reducing complexity

```
# Rule of Three
1 implementation → just write it
2 implementations → maybe extract common parts
3+ implementations → consider a trait/interface
```

## Dependency Evaluation

**Before adding a crate, consider:**
- Does the standard library have this? (prefer std)
- Is the dependency maintained? (check last commit, issues)
- What's the transitive dependency cost?
- Can we implement the needed subset ourselves in <100 lines?

**Acceptable reasons to add:**
- Security-critical code (crypto, parsing untrusted input)
- Complex domain (async runtime, serialization)
- Significant time savings with minimal dependency cost

**Poor reasons to add:**
- "It's popular"
- Saves 20 lines of straightforward code
- We only need 5% of its features

## API Surface

**Public API guidelines:**
- Minimize public surface - start private, expose when needed
- One obvious way to do things (not multiple paths to same result)
- Inputs should be validated at boundaries, trusted internally
- Return `Result` for operations that can fail; don't panic in library code

**Internal boundaries:**
- Modules expose a clean interface via `mod.rs`
- Implementation details stay private
- Cross-module communication through defined interfaces

## Error Handling

**Error philosophy:**
- Use `Result<T, E>` for recoverable errors
- Use `panic!` only for programmer errors (bugs), never for runtime conditions
- Errors should be actionable - tell the user what went wrong and how to fix it
- Prefer specific error types over `String` or generic `Error`

**Error granularity:**
- One error enum per module/domain is usually enough
- Don't create an error variant for every possible failure
- Group related failures when the caller doesn't need to distinguish them

## Trade-off Evaluation

When comparing approaches, evaluate:

| Criterion | Question |
|-----------|----------|
| Simplicity | Which is easier to understand? |
| Testability | Which is easier to test in isolation? |
| Maintainability | Which will be easier to modify later? |
| Performance | Does it matter here? (usually no) |
| Consistency | Which fits existing patterns? |

**Priority order:** Simplicity > Maintainability > Testability > Consistency > Performance

Performance is last because it rarely matters and is easy to optimize later. Premature optimization creates complexity that's hard to remove.

## Patterns in This Project

**Established patterns to follow:**
- Commands in `src/commands/` with one file per command
- Domain modules in `src/` (config, storage, recording, etc.)
- Tests in `tests/unit/` mirroring source structure
- Configuration via TOML in `~/.config/agr/`

**Patterns to avoid:**
- God modules that do everything
- Traits with single implementations
- Deep module hierarchies (prefer flat)
- Builder patterns for simple structs (just use struct literals)

## Checklist

Before finalizing a design:

- [ ] Is this the simplest approach that solves the problem?
- [ ] Are new modules justified by distinct responsibilities?
- [ ] Are abstractions backed by 3+ concrete uses?
- [ ] Are new dependencies truly necessary?
- [ ] Does the API surface stay minimal?
- [ ] Do errors help users fix problems?
- [ ] Does this fit existing project patterns?
