use std::ops::RangeInclusive;

use crate::Envelope;
use super::match_pattern::MatchPattern;

/// Pattern for matching maps.
#[derive(Debug, Clone)]
pub enum MapPattern {
    /// Matches any map.
    Any,
    /// Matches maps with a specific count of entries.
    Count(RangeInclusive<usize>),
}

impl MatchPattern for MapPattern {
    fn matches(&self, envelope: &Envelope) -> bool {
        if let Some(map) = envelope.subject().as_map() {
            match self {
                MapPattern::Any => true,
                MapPattern::Count(range) => range.contains(&map.len()),
            }
        } else {
            false
        }
    }
}
