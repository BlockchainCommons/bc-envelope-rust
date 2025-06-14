//! Non-recursive matcher for atomic patterns.
//!
//! This module provides atomic pattern matching that doesn't re-enter the VM,
//! preventing infinite recursion.

use crate::{
    Envelope,
    pattern::{Matcher, Path},
};

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
        Any => vec![vec![env.clone()]],
        None => vec![],
        Meta(meta) => match meta {
            crate::pattern::meta::MetaPattern::Search(_) => {
                panic!("SearchPattern should be compiled to Search instruction, not MatchPredicate");
            }
            _ => panic!(
                "non-atomic meta pattern used in MatchPredicate: {:?}",
                meta
            ),
        },
    }
}
