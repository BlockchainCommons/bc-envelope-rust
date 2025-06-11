use super::{Matcher, Path};
use crate::Envelope;

/// Pattern for matching boolean values.
#[derive(Debug, Clone)]
pub enum BoolPattern {
    /// Matches any boolean value.
    Any,
    /// Matches the specific boolean value.
    Exact(bool),
}

impl BoolPattern {
    /// Creates a new `BoolPattern` that matches any boolean value.
    pub fn any() -> Self { BoolPattern::Any }

    /// Creates a new `BoolPattern` that matches the specific boolean value.
    pub fn exact(value: bool) -> Self { BoolPattern::Exact(value) }
}

impl Matcher for BoolPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        let is_hit =
            envelope
                .extract_subject::<bool>()
                .ok()
                .map_or(false, |value| match self {
                    BoolPattern::Any => true,
                    BoolPattern::Exact(want) => value == *want,
                });

        if is_hit {
            vec![vec![envelope.clone()]]
        } else {
            vec![]
        }
    }
}
