//! Tiny Thompson-style VM for walking Gordian Envelope trees.
//!
//! The VM runs byte-code produced by `Pattern::compile` (implemented later).

use super::{Path, Pattern, atomic};
use crate::{EdgeType, Envelope};

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
        use crate::base::envelope::EnvelopeCase::*;
        match (self, env.case()) {
            (Axis::Subject, Node { subject, .. }) => {
                vec![(subject.clone(), EdgeType::Subject)]
            }
            (Axis::Assertion, Node { assertions, .. }) => assertions
                .iter()
                .cloned()
                .map(|a| (a, EdgeType::Assertion))
                .collect(),
            (Axis::Predicate, Assertion(a)) => {
                vec![(a.predicate().clone(), EdgeType::Predicate)]
            }
            (Axis::Object, Assertion(a)) => {
                vec![(a.object().clone(), EdgeType::Object)]
            }
            (Axis::Wrapped, Wrapped { envelope, .. }) => {
                vec![(envelope.clone(), EdgeType::Wrapped)]
            }
            _ => Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instr {
    MatchPredicate(usize),        // literals[idx].matches(env)
    Split { a: usize, b: usize }, // ε-split
    Jump(usize),                  // unconditional jump
    PushAxis(Axis),               // descend to children, one thread per child
    Pop,                          // pop one envelope from path
    Save,                         // emit current path
    Accept,                       // final accept
    Search { pat_idx: usize },    // NEW: search for pattern recursively
}

#[derive(Debug, Clone)]
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
    let mut stack =
        vec![Thread { pc: 0, env: root.clone(), path: vec![root.clone()] }];

    while let Some(mut th) = stack.pop() {
        loop {
            match prog.code[th.pc] {
                MatchPredicate(idx) => {
                    if atomic::atomic_paths(&prog.literals[idx], &th.env)
                        .is_empty()
                    {
                        break;
                    }
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
                Search { pat_idx } => {
                    // old SearchPattern::paths logic, but in-place and
                    // non-recursive
                    let inner = &prog.literals[pat_idx];

                    // 1) check current node
                    if !atomic::atomic_paths(inner, &th.env).is_empty() {
                        th.pc += 1;
                        continue; // success, stay on current envelope
                    }

                    // 2) otherwise walk children (same traversal as
                    //    Envelope::walk)
                    // Collect all children first, then push in reverse order to
                    // maintain the same traversal order as
                    // the original recursive implementation
                    let mut all_children = Vec::new();
                    for axis in [
                        Axis::Subject,
                        Axis::Assertion,
                        Axis::Predicate,
                        Axis::Object,
                        Axis::Wrapped,
                    ] {
                        for (child, _) in axis.children(&th.env) {
                            all_children.push(child);
                        }
                    }

                    // Push in reverse order so stack processes them in forward
                    // order
                    for child in all_children.into_iter().rev() {
                        let mut fork = th.clone();
                        fork.env = child.clone();
                        fork.path.push(child);
                        stack.push(fork); // revisit Search at the child
                    }
                    break; // this thread failed at current env
                }
            }
        }
    }
    out
}
