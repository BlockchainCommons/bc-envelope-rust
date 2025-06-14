use std::ops::RangeInclusive;

use crate::{
    Envelope,
    pattern::{Matcher, Path},
};

/// Pattern for matching node envelopes.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum NodePattern {
    /// Matches any node.
    Any,
    /// Matches a node with the specified count of assertions.
    AssertionsCount(RangeInclusive<usize>),
}

impl NodePattern {
    /// Creates a new `NodePattern` that matches any node.
    pub fn any() -> Self { NodePattern::Any }

    /// Creates a new `NodePattern` that matches a node with the specified count
    /// of assertions.
    pub fn assertions_count_range(range: RangeInclusive<usize>) -> Self {
        NodePattern::AssertionsCount(range)
    }

    /// Creates a new `NodePattern` that matches a node with exactly the
    /// specified number of assertions.
    pub fn assertions_count(count: usize) -> Self {
        NodePattern::AssertionsCount(count..=count)
    }
}

impl Matcher for NodePattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        if !envelope.is_node() {
            return vec![];
        }

        let is_hit = match self {
            NodePattern::Any => true,
            NodePattern::AssertionsCount(range) => {
                range.contains(&envelope.assertions().len())
            }
        };

        if is_hit {
            vec![vec![envelope.clone()]]
        } else {
            vec![]
        }
    }
}
