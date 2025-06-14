//! Tiny Thompson-style VM for walking Gordian Envelope trees.
//!
//! The VM runs byte-code produced by `Pattern::compile` (implemented later).

use super::{Matcher, Path, Pattern};
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
                    if !prog.literals[idx].matches(&th.env) {
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
            }
        }
    }
    out
}
