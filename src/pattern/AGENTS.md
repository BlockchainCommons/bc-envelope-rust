# AGENTS.md

Finish the `pattern` subsystem so that `RepeatPattern` and `CapturePattern` are fully supported, and all crate tests pass.

What follows is everything you need:

# High-level goal

Turn the existing pattern DSL into a complete back-tracking engine that supports

- Equivalent functionality to regex `*`, `+`, `?`, `{n,m}` quantifiers (greedy, lazy, possessive)
- Named captures (placeholder for now – just emit paths).

# Big picture

We compile the Pattern AST to a small Thompson-NFA-like byte-code (in `src/pattern/vm.rs`) and execute it against an Envelope tree. All mutations are additive; existing APIs continue to work.

# Tests

These tests are hanging:

```bash
cargo test --test 'pattern_tests_repeat'
```

This runs all pattern tests, including the hanging ones:

```bash
cargo test --test 'pattern_tests*'
```
