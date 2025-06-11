use std::ops::RangeInclusive;

use crate::Envelope;
use super::matcher::Matcher;

/// Pattern for matching node envelopes.
#[derive(Debug, Clone)]
pub enum NodePattern {
    /// Matches any node.
    Any,
    /// Matches a node with the specified count of assertions.
    AssertionsCount(RangeInclusive<usize>),
}

impl MatchPattern for NodePattern {
    fn matches(&self, envelope: &Envelope) -> bool {
        if !envelope.is_node() {
            return false;
        }
        match self {
            NodePattern::Any => true,
            NodePattern::AssertionsCount(range) => {
                range.contains(&envelope.assertions().len())
            }
        }
    }
}
