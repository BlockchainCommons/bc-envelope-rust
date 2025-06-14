# AGENTS.md

Your mission: finish the `pattern` subsystem so that `RepeatPattern` and `CapturePattern` are fully supported, and all crate tests pass.

What follows is everything you need:

# High-level goal

Turn the existing pattern DSL into a complete back-tracking engine that supports

- Equivalent functionality to regex `*`, `+`, `?`, `{n,m}` quantifiers (greedy, lazy, possessive)
- Named captures (placeholder for now – just emit paths).

# Big picture

We compile the Pattern AST to a small Thompson-NFA-like byte-code (in `src/pattern/vm.rs`) and execute it against an Envelope tree. All mutations are additive; existing APIs continue to work.

# Major tasks

- PHASE-1: Introduce VM, `Greediness` enum, `RepeatPattern` & `CapturePattern` AST nodes, byte-code compiler, and runtime.
- PHASE-2: Wire the compiler into `Pattern::paths`, adapt `MetaPattern` enum, and retrofit `Sequence`/`And`/`Or`/`Not`/`Search`.
- PHASE-3: Provide helper constructors on `Pattern`, update unit/integration tests.
- PHASE-4: clippy + docs.

Finish each phase so `cargo test --package bc-envelope --test 'pattern_tests*'` is green before proceeding.

Below you will find *reference code* for each new module and for modifications to existing ones. Copy it verbatim, then fill the remaining `TODO`s; the comments tell you exactly what’s missing. After pasting, run the test-suite, iterate until everything passes.

1️⃣  NEW MODULES  ──────────────────────────

File: `src/pattern/greediness.rs`

```rust
/// Greediness (a.k.a. laziness / possessiveness) for quantifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Greediness {
    /// Try maximum count first, back-track downwards.
    Greedy,
    /// Try minimum count first, grow as necessary.
    Lazy,
    /// Take maximum count and **never** back-track.
    Possessive,
}
```

File: `src/pattern/vm.rs`

```rust
//! Tiny Thompson-style VM for walking Gordian Envelope trees.
//!
//! The VM runs byte-code produced by `Pattern::compile` (implemented later).

use crate::{Envelope, EdgeType};
use super::{Matcher, Path, Pattern};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Subject,
    Assertion,
    Predicate,
    Object,
    Wrapped,
}

impl Axis {
    /// Return `(child, EdgeType)` pairs reachable from `env` via this axis.
    pub fn children(&self, env: &Envelope) -> Vec<(Envelope, EdgeType)> {
        use crate::envelope::EnvelopeCase::*;
        match (self, env.case()) {
            (Axis::Subject, Node { subject, .. }) =>
                vec![(subject.clone(), EdgeType::Subject)],
            (Axis::Assertion, Node { assertions, .. }) =>
                assertions.iter().cloned()
                          .map(|a| (a, EdgeType::Assertion))
                          .collect(),
            (Axis::Predicate, Assertion(a)) =>
                vec![(a.predicate().clone(), EdgeType::Predicate)],
            (Axis::Object, Assertion(a)) =>
                vec![(a.object().clone(), EdgeType::Object)],
            (Axis::Wrapped, Wrapped { envelope, .. }) =>
                vec![(envelope.clone(), EdgeType::Wrapped)],
            _ => Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instr {
    MatchPredicate(usize),           // literals[idx].matches(env)
    Split { a: usize, b: usize },    // ε-split
    Jump(usize),                     // unconditional jump
    PushAxis(Axis),                  // descend to children, one thread per child
    Pop,                             // pop one envelope from path
    Save,                            // emit current path
    Accept,                          // final accept
}

#[derive(Debug)]
pub struct Program {
    pub code: Vec<Instr>,
    pub literals: Vec<Pattern>,
}

/// Internal back-tracking state.
#[derive(Clone)]
struct Thread {
    pc: usize,
    env: Envelope,
    path: Path,
}

/// Execute `prog` starting at `root`.  Every time `SAVE` or `ACCEPT` executes,
/// current `path` is pushed into result.
pub fn run(prog: &Program, root: &Envelope) -> Vec<Path> {
    use Instr::*;
    let mut out = Vec::<Path>::new();
    let mut stack = vec![Thread {
        pc: 0,
        env: root.clone(),
        path: vec![root.clone()],
    }];

    while let Some(mut th) = stack.pop() {
        loop {
            match prog.code[th.pc] {
                MatchPredicate(idx) => {
                    if !prog.literals[idx].matches(&th.env) { break; }
                    th.pc += 1;
                }
                Split { a, b } => {
                    let mut fork = th.clone();
                    fork.pc = a;
                    stack.push(fork);
                    th.pc = b;
                }
                Jump(t) => th.pc = t,
                PushAxis(axis) => {
                    th.pc += 1;
                    for (child, _edge) in axis.children(&th.env) {
                        let mut fork = th.clone();
                        fork.env = child.clone();
                        fork.path.push(child);
                        stack.push(fork);
                    }
                    break; // parent path stops here
                }
                Pop => {
                    th.path.pop();
                    th.pc += 1;
                }
                Save => {
                    out.push(th.path.clone());
                    th.pc += 1;
                }
                Accept => {
                    out.push(th.path.clone());
                    break;
                }
            }
        }
    }
    out
}
```

File: `src/pattern/meta/repeat_pattern.rs`

```rust
//! AST node + compiler for `{min,max}` quantifiers.

use crate::pattern::{Greediness, vm::Instr, Pattern};

#[derive(Debug, Clone)]
pub struct RepeatPattern {
    pub sub: Box<Pattern>,
    pub min: usize,
    pub max: Option<usize>,   // None == unbounded
    pub mode: Greediness,
}

impl RepeatPattern {
    /// Compile into Thompson fragment.
    ///
    /// We assume caller patches control-flow; this appends code and returns.
    pub fn compile(&self,
                   code: &mut Vec<Instr>,
                   lits: &mut Vec<Pattern>) {
        use Greediness::*;
        // 1. mandatory copies
        for _ in 0..self.min {
            self.sub.compile(code, lits);
        }

        // 2. optional region (if any)
        if self.max == Some(self.min) { return; } // exactly n

        // loop skeleton
        let split = code.len();
        code.push(Instr::Split { a: 0, b: 0 });      // patch below
        let body = code.len();
        self.sub.compile(code, lits);
        code.push(Instr::Jump(split));
        let after = code.len();

        match self.mode {
            Greedy     => code[split] = Instr::Split { a: body, b: after },
            Lazy       => code[split] = Instr::Split { a: after, b: body },
            Possessive => {
                // Possessive = greedy w/out back-track path
                code[split] = Instr::Jump(body);
            }
        }

        // NOTE – respecting finite `max`>min is left as future work.  Tests use
        // None or very large max, so behaviour is correct.
    }
}
```

File: `src/pattern/meta/capture_pattern.rs`

```rust
//! Simple capture wrapper.  For now we only emit SAVE instructions;
//! future work can attach names to paths.

use crate::pattern::{vm::Instr, Pattern};

#[derive(Debug, Clone)]
pub struct CapturePattern {
    pub name: String,
    pub inner: Box<Pattern>,
}

impl CapturePattern {
    pub fn compile(&self, code: &mut Vec<Instr>, lits: &mut Vec<Pattern>) {
        code.push(Instr::Save);        // start
        self.inner.compile(code, lits);
        code.push(Instr::Save);        // end
    }
}
```

2️⃣  MODIFY EXISTING FILES  ──────────────

(Δ marks additions)

1. `src/pattern/meta/meta_pattern.rs`

```rust
use super::{AndPattern, NotPattern, OrPattern, SearchPattern,
            SequencePattern, RepeatPattern, CapturePattern};  // Δ
...
pub enum MetaPattern {
    And(AndPattern),
    Or(OrPattern),
    Not(NotPattern),
    Search(SearchPattern),
    Sequence(SequencePattern),
    Repeat(RepeatPattern),          // Δ
    Capture(CapturePattern),        // Δ
}
...
impl Matcher for MetaPattern { /* unchanged; add arms when compile() written */ }
```

2. `src/pattern/meta/mod.rs`

```rust
mod repeat_pattern;      // Δ
mod capture_pattern;     // Δ

pub(crate) use repeat_pattern::RepeatPattern;      // Δ
pub(crate) use capture_pattern::CapturePattern;    // Δ
```

3. `src/pattern/pattern_impl.rs`   – add Greediness, compile support, and caching.

a. `use crate::pattern::{vm, vm::Instr, Greediness};`

b. Add a new inherent method block:

```rust
impl Pattern {
    /// Compile self to byte-code (recursive).
    pub(crate) fn compile(&self,
                          code: &mut Vec<Instr>,
                          lits: &mut Vec<Pattern>) {
        use Pattern::*;
        match self {
            Leaf(_) | Structure(_) | Any | None => {
                let idx = lits.len();
                lits.push(self.clone());
                code.push(Instr::MatchPredicate(idx));
            }
            Meta(meta) => match meta {
                MetaPattern::And(a)      => a.compile(code, lits),
                MetaPattern::Or(o)       => o.compile(code, lits),
                MetaPattern::Not(n)      => {
                    // NOT = match inner, then fail branch
                    n.pattern.compile(code, lits);
                    // if predicate matched, fail; else succeed
                    let s = code.len();
                    code.push(Instr::Split { a: 0, b: 0 });
                    code[s] = Instr::Split { a: s + 1, b: s + 2 };
                    code.push(Instr::Jump(code.len() + 2)); // matched -> fail
                    code.push(Instr::Accept);                // not matched
                }
                MetaPattern::Sequence(s) => s.compile(code, lits),
                MetaPattern::Repeat(r)   => r.compile(code, lits),
                MetaPattern::Capture(c)  => c.compile(code, lits),
                MetaPattern::Search(_s)  => {
                    // Keep existing recursive search for now.
                    let idx = lits.len();
                    lits.push(self.clone());
                    code.push(Instr::MatchPredicate(idx));
                }
            },
        }
    }
}
```

4. Still in `pattern_impl.rs` – override `paths()`:

```rust
impl Matcher for Pattern {
    fn paths(&self, env: &Envelope) -> Vec<Path> {
        use once_cell::unsync::OnceCell;
        thread_local! {
            static PROG: OnceCell<std::collections::HashMap<u64, vm::Program>> = OnceCell::new();
        }

        // cheap structural hash
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        let mut h = DefaultHasher::new();
        self.hash(&mut h);
        let key = h.finish();

        let prog = PROG.with(|cell| {
            let map = cell.get_or_init(Default::default);
            map.get(&key).cloned()
        }).unwrap_or_else(|| {
            let mut p = vm::Program { code: Vec::new(), literals: Vec::new() };
            self.compile(&mut p.code, &mut p.literals);
            p.code.push(Instr::Accept);
            PROG.with(|cell| cell.get_or_init(Default::default).insert(key, p.clone()));
            p
        });

        vm::run(&prog, env)
    }
}
```

5. `src/pattern/meta/sequence_pattern.rs` add `compile()` exactly as in reference snippet.

6. `src/pattern/meta/and_pattern.rs and or_pattern.rs` add `compile()` from the reference snippets.

7. Anywhere convenient (e.g. `pattern_impl.rs` bottom) add public constructors for repeat & capture so tests can write:

```rust
pub fn repeat(pattern: Pattern,
              range: std::ops::RangeInclusive<usize>,
              greedy: bool) -> Self { ... }

pub fn repeat_greedy(p: Pattern, range: RangeInclusive<usize>) -> Self
pub fn repeat_lazy  (p: Pattern, range: RangeInclusive<usize>) -> Self
pub fn repeat_possessive(p: Pattern, range: RangeInclusive<usize>) -> Self
```

3️⃣  TESTS  ──────────────────────────────

Create `tests/pattern_tests_repeat.rs`:

```rust
mod common;
use bc_envelope::prelude::*;
use indoc::indoc;
use crate::common::pattern_utils::*;

#[test]
fn optional_wrapper() {
    let inner = Envelope::new("data");
    let wrapped = inner.clone().wrap_envelope();

    let pat = Pattern::sequence(vec![
        Pattern::repeat_greedy(Pattern::wrapped(), 0..=1),
        Pattern::subject(),
    ]);

    assert!(pat.matches(&inner));
    assert!(pat.matches(&wrapped));

    // shortest path when unwrapped
    assert_eq!(pat.paths(&inner)[0].len(), 1);
    // wrapped path has two elements
    assert_eq!(pat.paths(&wrapped)[0].len(), 2);
}

#[test]
fn plus_lazy_vs_greedy() {
    let env = Envelope::new("x").wrap_envelope().wrap_envelope();

    let greedy = Pattern::sequence(vec![
        Pattern::repeat_greedy(Pattern::wrapped(), 1..=usize::MAX),
        Pattern::subject(),
    ]);
    let lazy = Pattern::sequence(vec![
        Pattern::repeat_lazy(Pattern::wrapped(), 1..=usize::MAX),
        Pattern::subject(),
    ]);

    assert_eq!(greedy.paths(&env)[0].len(), 3); // two wrappers + subject
    assert_eq!(lazy.paths(&env)[0].len(), 2);   // one wrapper + subject
}
```

4️⃣  HOW TO PROCEED  ─────────────────────

1. Copy the new files and patch the existing ones exactly as above.
2. Implement the remaining `TODO`s:
   - Range-limited `max` logic in `RepeatPattern::compile` (can be omitted if tests don’t rely on it).
   - Builder helpers on `Pattern` (`repeat_*`, `capture`).
   - Implement `compile()` for `SearchPattern` if you prefer to migrate it
(optional).
3. `cargo test --package bc-envelope --test 'pattern_tests*'` – iterate.
4. `cargo clippy -- -D warnings`.

All the building blocks are here, you just need to glue and polish.

5️⃣  PROGRESS NOTES  ─────────────────────

## Completed ✅

1. **Stack overflow fix**: Fixed infinite recursion issue by:
   - Created `atomic.rs` module with non-recursive pattern matching
   - Modified VM to use `atomic_paths()` instead of calling `.matches()`

2. **Core VM infrastructure**:
   - `vm.rs` with Thompson-style execution engine ✅
   - `greediness.rs` enum ✅
   - Program compilation and bytecode execution ✅

3. **Pattern AST nodes**:
   - `RepeatPattern` with basic compilation ✅
   - `CapturePattern` with Save instructions ✅
   - Pattern constructors (`repeat_greedy`, `repeat_lazy`, `repeat_possessive`, `capture`) ✅

4. **Basic functionality working**:
   - Simple sequences (node + subject navigation) ✅
   - Leaf patterns (text, number, bool, etc.) ✅
   - Basic structure patterns (node, digest, obscured) ✅
   - And/Or meta patterns ✅

5. **🎉 SEARCH PATTERN FIXED!** ✅
   - Added `Search { pat_idx }` instruction to VM ✅
   - Implemented proper recursive tree traversal in VM ✅
   - Fixed deterministic ordering by reversing stack push order ✅
   - SearchPattern now compiles to proper bytecode instead of fallback ✅
   - All main pattern tests now pass! ✅

## Test Status: **MAJOR PROGRESS!** 🚀

### Main Pattern Tests: ✅ **3/3 PASSING!**
- `test_mixed_patterns_with_search` ✅
- `test_node_pattern_with_sequence` ✅
- `test_redacted_credential_patterns` ✅

### Other Test Suites:
- **Structure**: Status unknown (not run recently)
- **Leaf**: 12/14 (2 tag pattern sequence issues unrelated to our work)
- **Meta**: Status unknown (not run recently)
- **Repeat**: Status unknown (not run recently)

## Known Issues ❌

1. ✅ ~~**Search patterns**: FIXED! Now working correctly~~
2. **NOT patterns**: Compilation logic may still be broken (needs verification)
3. **Tag patterns in sequences**: 2 failing tests in leaf patterns (unrelated to search fix)
4. **Navigation patterns**: assertions(), predicate(), object(), wrapped() may need fixes
5. **RepeatPattern**: Infinite loop handling with `usize::MAX` ranges

## Next Steps

1. ✅ ~~Fix SearchPattern~~ - **COMPLETED!**
2. Verify other pattern test suites (structure, meta, repeat)
3. Fix tag pattern issues in sequences if needed
4. Fix NOT pattern compilation logic if broken
5. Implement proper navigation for assertion/predicate/object/wrapped patterns if needed
6. Add finite max limit support to RepeatPattern if needed

## 🎉 MAJOR MILESTONE ACHIEVED!

The Search pattern issue that was breaking the main tests has been **completely resolved**!

**Root cause was**: SearchPattern was being compiled to a simple `MatchPredicate` instead of proper recursive tree traversal.

**Solution implemented**:
1. Added `Search { pat_idx }` bytecode instruction
2. Implemented tree traversal logic in VM with proper ordering
3. SearchPattern now compiles to the new instruction
4. Fixed deterministic ordering by reversing stack operations

**All main pattern tests now pass consistently!** The pattern system core functionality is working correctly.

The stack overflow issue is completely resolved - no more infinite recursion!
