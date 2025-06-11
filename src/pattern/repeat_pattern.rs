use std::ops::RangeInclusive;

use super::{match_pattern::MatchPattern, pattern::Pattern};
use crate::Envelope;

/// A pattern that matches a repeated element.
#[derive(Debug, Clone)]
pub struct RepeatPattern {
    pub(crate) element: Box<Pattern>,
    pub(crate) range: RangeInclusive<usize>,
}

impl MatchPattern for RepeatPattern {
    fn matches(&self, envelope: &Envelope) -> bool {
        // Placeholder implementation - would need to be updated
        if let Some(CBORCase::Array(array)) = envelope.subject().case() {
            self.range.contains(&array.len())
                && array.iter().all(|_item| {
                    // Check each item against the element pattern
                    // Note: This is a simplification and would need actual
                    // implementation
                    true
                })
        } else {
            false
        }
    }
}

use dcbor::prelude::CBORCase;
