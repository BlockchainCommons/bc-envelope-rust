use std::ops::RangeInclusive;

use crate::Envelope;
use super::matcher::Matcher;

/// Pattern for matching arrays.
#[derive(Debug, Clone)]
pub enum ArrayPattern {
    /// Matches any array.
    Any,
    /// Matches arrays with a specific count of elements.
    Count(RangeInclusive<usize>),
}

impl MatchPattern for ArrayPattern {
    fn matches(&self, envelope: &Envelope) -> bool {
        if let Some(array) = envelope.subject().as_array() {
            match self {
                ArrayPattern::Any => true,
                ArrayPattern::Count(range) => range.contains(&array.len()),
            }
        } else {
            false
        }
    }
}
