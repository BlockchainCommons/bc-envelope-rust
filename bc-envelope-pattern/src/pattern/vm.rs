//! Tiny Thompson-style VM for walking Gordian Envelope trees.
//!
//! The VM runs byte-code produced by `Pattern::compile` (implemented later).

use super::{Greediness, Matcher, Path, Pattern};
use crate::{EdgeType, Envelope};
use bc_components::DigestProvider;

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
        use bc_envelope::base::envelope::EnvelopeCase::*;
        // println!("Axis::children called with axis: {:?}", self);
        // println!("Case: {:?}", env.case());
        // println!("Envelope: {}", env.format_flat());
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
            (Axis::Wrapped, Node { subject, .. }) => {
                if subject.is_wrapped() {
                    vec![(
                        subject.unwrap_envelope().unwrap(),
                        EdgeType::Wrapped,
                    )]
                } else {
                    vec![]
                }
            }
            (Axis::Wrapped, Wrapped { envelope, .. }) => {
                vec![(envelope.clone(), EdgeType::Wrapped)]
            }
            _ => Vec::new(),
        }
    }
}

/// Bytecode instructions for the pattern VM.
#[derive(Debug, Clone)]
pub enum Instr {
    /// Match predicate: `literals[idx].matches(env)`
    MatchPredicate(usize),
    /// Match structure: use `literals[idx].paths(env)` for structure patterns
    MatchStructure(usize),
    /// ε-split: fork execution to `a` and `b`
    Split { a: usize, b: usize },
    /// Unconditional jump to instruction at index
    Jump(usize),
    /// Descend to children via axis, one thread per child
    PushAxis(Axis),
    /// Pop one envelope from the path
    Pop,
    /// Emit current path
    Save,
    /// Final accept, emit current path and halt thread
    Accept,
    /// Recursively search for pattern at `pat_idx`
    Search { pat_idx: usize },
    /// Save current path and start new sequence from last envelope
    ExtendSequence,
    /// Combine saved path with current path for final result
    CombineSequence,
    /// Navigate to subject of current envelope
    NavigateSubject,
    /// Match only if pattern at `pat_idx` does not match
    NotMatch { pat_idx: usize },
    /// Repeat a sub pattern according to range and greediness
    Repeat {
        pat_idx: usize,
        min: usize,
        max: Option<usize>,
        mode: Greediness,
    },
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
    /// Stack of saved paths for nested sequence patterns
    saved_paths: Vec<Path>,
}

/// Match atomic patterns without recursion into the VM.
///
/// This function handles only the patterns that are safe to use in
/// MatchPredicate instructions - Leaf, Structure, Any, and None patterns. Meta
/// patterns should never be passed to this function as they need to be compiled
/// to bytecode.
///
/// For SearchPattern, we provide a temporary fallback that uses the old
/// recursive implementation until proper bytecode compilation is implemented.
#[allow(clippy::panic)]
pub(crate) fn atomic_paths(
    p: &crate::pattern::Pattern,
    env: &Envelope,
) -> Vec<Path> {
    use crate::pattern::Pattern::*;
    match p {
        Leaf(l) => l.paths(env),
        Structure(s) => s.paths(env),
        Meta(meta) => match meta {
            crate::pattern::meta::MetaPattern::Any(a) => a.paths(env),
            crate::pattern::meta::MetaPattern::None(n) => n.paths(env),
            crate::pattern::meta::MetaPattern::Search(_) => {
                panic!(
                    "SearchPattern should be compiled to Search instruction, not MatchPredicate"
                );
            }
            _ => panic!(
                "non-atomic meta pattern used in MatchPredicate: {:?}",
                meta
            ),
        },
    }
}

fn repeat_paths(
    pat: &Pattern,
    env: &Envelope,
    path: &Path,
    min: usize,
    max: Option<usize>,
    mode: Greediness,
) -> Vec<(Envelope, Path)> {
    let mut states: Vec<Vec<(Envelope, Path)>> = vec![vec![(env.clone(), path.clone())]];
    let bound = max.unwrap_or(usize::MAX);
    for _ in 0..bound {
        let mut next = Vec::new();
        for (e, pth) in states.last().unwrap().iter() {
            for sub_path in pat.paths(e) {
                if let Some(last) = sub_path.last() {
                    if last.digest() == e.digest() {
                        continue;
                    }
                    let mut combined = pth.clone();
                    if sub_path.first() == Some(e) {
                        combined.extend(sub_path.iter().skip(1).cloned());
                    } else {
                        combined.extend(sub_path.iter().cloned());
                    }
                    next.push((last.clone(), combined));
                }
            }
        }
        if next.is_empty() {
            break;
        }
        states.push(next);
    }

    let max_possible = states.len() - 1;
    let max_allowed = bound.min(max_possible);
    if max_allowed < min {
        return Vec::new();
    }

    let counts: Vec<usize> = match mode {
        Greediness::Greedy => (min..=max_allowed).rev().collect(),
        Greediness::Lazy => (min..=max_allowed).collect(),
        Greediness::Possessive => vec![max_allowed],
    };

    let mut out = Vec::new();
    for c in counts {
        if let Some(list) = states.get(c) {
            out.extend(list.clone());
        }
    }
    out
}

/// Execute `prog` starting at `root`.  Every time `SAVE` or `ACCEPT` executes,
/// current `path` is pushed into result.
/// Execute a single thread until it halts. Returns true if any paths were
/// produced.
fn run_thread(prog: &Program, start: Thread, out: &mut Vec<Path>) -> bool {
    use Instr::*;
    let mut produced = false;
    let mut stack = vec![start];

    while let Some(mut th) = stack.pop() {
        loop {
            match prog.code[th.pc] {
                MatchPredicate(idx) => {
                    if atomic_paths(&prog.literals[idx], &th.env).is_empty() {
                        break;
                    }
                    th.pc += 1;
                }
                MatchStructure(idx) => {
                    // Use the structure pattern's direct matcher, not the
                    // compiled pattern
                    let structure_paths =
                        if let crate::pattern::Pattern::Structure(sp) =
                            &prog.literals[idx]
                        {
                            // Call the structure pattern's direct paths method
                            sp.paths(&th.env)
                        } else {
                            panic!(
                                "MatchStructure used with non-structure pattern"
                            );
                        };

                    if structure_paths.is_empty() {
                        break;
                    }

                    th.pc += 1; // Advance to next instruction

                    // Spawn a new thread for each path found by the structure
                    // pattern
                    for (i, structure_path) in
                        structure_paths.into_iter().enumerate()
                    {
                        if i == 0 {
                            // Use the first path for the current thread
                            th.path = structure_path.clone();
                            if let Some(last_env) = structure_path.last() {
                                th.env = last_env.clone();
                            }
                        } else {
                            // Spawn new threads for the remaining paths
                            let mut fork = th.clone();
                            fork.path = structure_path.clone();
                            if let Some(last_env) = structure_path.last() {
                                fork.env = last_env.clone();
                            }
                            stack.push(fork);
                        }
                    }
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
                    produced = true;
                    th.pc += 1;
                }
                Accept => {
                    out.push(th.path.clone());
                    produced = true;
                    break;
                }
                Search { pat_idx } => {
                    // old SearchPattern::paths logic, but in-place and
                    // non-recursive
                    let inner = &prog.literals[pat_idx];

                    // 1) check current node and get the actual paths it matches
                    let found_paths = match inner {
                        crate::pattern::Pattern::Leaf(_) => {
                            inner.paths(&th.env)
                        }
                        crate::pattern::Pattern::Structure(_) => {
                            inner.paths(&th.env)
                        }
                        crate::pattern::Pattern::Meta(_) => {
                            inner.paths(&th.env)
                        }
                    };

                    // If we found any paths, emit them by extending the current
                    // path
                    if !found_paths.is_empty() {
                        produced = true;
                        for found_path in found_paths {
                            // Special case: if the found path is just the
                            // current envelope,
                            // emit the current path instead of extending it
                            if found_path.len() == 1 && found_path[0] == th.env
                            {
                                out.push(th.path.clone());
                            } else {
                                // Extend current path with the found path
                                let mut result_path = th.path.clone();
                                result_path.extend(found_path);
                                out.push(result_path);
                            }
                        }
                    }

                    // 2) always walk children (same traversal as
                    //    Envelope::walk)
                    // Collect all children first, then push in reverse order to
                    // maintain the same traversal order as
                    // the original recursive implementation
                    // Build child list following the same structure order as
                    // `Envelope::walk_structure`. This ensures every envelope
                    // is visited exactly once.
                    let mut all_children = Vec::new();
                    use bc_envelope::base::envelope::EnvelopeCase::*;
                    match th.env.case() {
                        Node { subject, assertions, .. } => {
                            all_children.push(subject.clone());
                            for assertion in assertions {
                                all_children.push(assertion.clone());
                            }
                        }
                        Wrapped { envelope, .. } => {
                            all_children.push(envelope.clone());
                        }
                        Assertion(assertion) => {
                            all_children.push(assertion.predicate().clone());
                            all_children.push(assertion.object().clone());
                        }
                        _ => {}
                    }

                    // Push child threads in reverse order so stack processes
                    // them in forward order
                    for child in all_children.into_iter().rev() {
                        let mut fork = th.clone();
                        fork.env = child.clone();
                        fork.path.push(child);
                        // fork continues with same PC to re-execute Search at
                        // child
                        stack.push(fork);
                    }

                    // This thread is done - either it emitted results or it
                    // didn't
                    break;
                }
                ExtendSequence => {
                    // Save the current path and switch to the last envelope for
                    // the rest of the sequence
                    if let Some(last_env) = th.path.last().cloned() {
                        th.saved_paths.push(th.path.clone());
                        th.env = last_env.clone();
                        th.path = vec![last_env]; // Start fresh path from the last envelope
                    }
                    th.pc += 1;
                }
                CombineSequence => {
                    // Combine saved path with current path, extending the saved
                    // path
                    if let Some(saved_path) = th.saved_paths.pop() {
                        let mut combined = saved_path.clone();

                        // If the current path starts with the same envelope as
                        // the saved path ends with,
                        // skip the first element to avoid duplication.
                        // Otherwise, append the whole current path.
                        if let (Some(saved_last), Some(current_first)) =
                            (saved_path.last(), th.path.first())
                        {
                            if saved_last == current_first {
                                // Skip first element to avoid duplication
                                combined
                                    .extend(th.path.iter().skip(1).cloned());
                            } else {
                                // Append whole current path
                                combined.extend(th.path.iter().cloned());
                            }
                        } else {
                            // Append whole current path if one of the paths is
                            // empty
                            combined.extend(th.path.iter().cloned());
                        }

                        th.path = combined;
                    }
                    th.pc += 1;
                }
                Repeat { pat_idx, min, max, mode } => {
                    let pat = &prog.literals[pat_idx];
                    let results = repeat_paths(pat, &th.env, &th.path, min, max, mode);
                    if results.is_empty() {
                        break;
                    }
                    // Try each repetition count in order. `run_thread` fully
                    // explores all branches for that count and returns `true`
                    // if it yields any paths. Once one count succeeds we stop
                    // trying further counts, emulating regex greedy/lazy
                    // semantics while still returning all matching paths for
                    // the chosen count.
                    let next_pc = th.pc + 1;
                    let mut success = false;
                    for (env_after, path_after) in results {
                        let mut fork = th.clone();
                        fork.pc = next_pc;
                        fork.env = env_after;
                        fork.path = path_after;
                        if run_thread(prog, fork, out) {
                            produced = true;
                            success = true;
                            break;
                        }
                    }
                    if !success {
                        // None of the repetition counts allowed the rest to match
                    }
                    break;
                }
                NavigateSubject => {
                    // If the current envelope is a node, navigate to its
                    // subject and update the path.
                    if th.env.is_node() {
                        let subject = th.env.subject();
                        th.env = subject.clone();
                        th.path.push(subject);
                    }
                    th.pc += 1;
                }
                NotMatch { pat_idx } => {
                    // Check if the pattern matches. If it doesn't match, the
                    // NOT pattern succeeds. If it does
                    // match, the NOT pattern fails and we kill this thread.

                    // For atomic patterns, use atomic_paths for efficiency
                    let pattern = &prog.literals[pat_idx];
                    let pattern_matches = match pattern {
                        crate::pattern::Pattern::Leaf(_) => {
                            pattern.matches(&th.env)
                        }
                        crate::pattern::Pattern::Structure(_) => {
                            pattern.matches(&th.env)
                        }
                        crate::pattern::Pattern::Meta(_) => {
                            pattern.matches(&th.env)
                        }
                    };

                    if pattern_matches {
                        // Inner pattern matches, so NOT pattern fails - kill
                        // this thread
                        break;
                    } else {
                        // Inner pattern doesn't match, so NOT pattern succeeds
                        // - continue
                        th.pc += 1;
                    }
                }
            }
        }
    }
    produced
}

/// Execute `prog` starting at `root`.  Every time `SAVE` or `ACCEPT` executes,
/// the current `path` is pushed into the result.
pub fn run(prog: &Program, root: &Envelope) -> Vec<Path> {
    let mut out = Vec::new();
    let start = Thread {
        pc: 0,
        env: root.clone(),
        path: vec![root.clone()],
        saved_paths: Vec::new(),
    };
    run_thread(prog, start, &mut out);
    out
}
