use std::ops::RangeInclusive;

use crate::{
    Envelope,
    pattern::{Matcher, Path},
};

/// Pattern for matching arrays.
#[derive(Debug, Clone)]
pub enum ArrayPattern {
    /// Matches any array.
    Any,
    /// Matches arrays with a specific count of elements.
    Count(RangeInclusive<usize>),
}

impl ArrayPattern {
    /// Creates a new `ArrayPattern` that matches any array.
    pub fn any() -> Self { ArrayPattern::Any }

    /// Creates a new `ArrayPattern` that matches arrays with a specific count
    /// of elements.
    pub fn count(range: RangeInclusive<usize>) -> Self {
        ArrayPattern::Count(range)
    }

    /// Creates a new `ArrayPattern` that matches arrays with exactly the
    /// specified count of elements.
    pub fn exact_count(count: usize) -> Self {
        ArrayPattern::Count(count..=count)
    }
}

impl Matcher for ArrayPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        if let Some(array) = envelope.subject().as_array() {
            match self {
                ArrayPattern::Any => vec![vec![envelope.clone()]],
                ArrayPattern::Count(range) => {
                    if range.contains(&array.len()) {
                        vec![vec![envelope.clone()]]
                    } else {
                        vec![]
                    }
                }
            }
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use dcbor::prelude::*;

    use super::*;
    use crate::Envelope;

    #[test]
    fn test_array_pattern_any() {
        // Create a CBOR array directly
        let cbor_array = vec![CBOR::from(1), CBOR::from(2), CBOR::from(3)];
        let envelope = Envelope::new(cbor_array);
        let pattern = ArrayPattern::any();
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec![envelope.clone()]);

        // Test with non-array envelope
        let text_envelope = Envelope::new("test");
        let paths = pattern.paths(&text_envelope);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_array_pattern_count() {
        // Create a CBOR array directly
        let cbor_array = vec![CBOR::from(1), CBOR::from(2), CBOR::from(3)];
        let envelope = Envelope::new(cbor_array);

        // Test exact count
        let pattern = ArrayPattern::exact_count(3);
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);

        // Test count range
        let pattern = ArrayPattern::count(2..=4);
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);

        // Test count mismatch
        let pattern = ArrayPattern::exact_count(5);
        let paths = pattern.paths(&envelope);
        assert!(paths.is_empty());
    }
}
