use std::ops::RangeInclusive;

use crate::{
    pattern::{compile_as_atomic, leaf::LeafPattern, vm::Instr, Compilable, Matcher, Path}, Envelope, Pattern
};

/// Pattern for matching arrays.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
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
    pub fn range_count(range: RangeInclusive<usize>) -> Self {
        ArrayPattern::Count(range)
    }

    /// Creates a new `ArrayPattern` that matches arrays with exactly the
    /// specified count of elements.
    pub fn count(count: usize) -> Self { ArrayPattern::Count(count..=count) }
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

impl Compilable for ArrayPattern {
    fn compile(&self, code: &mut Vec<Instr>, literals: &mut Vec<Pattern>) {
        compile_as_atomic(
            &Pattern::Leaf(LeafPattern::Array(
                self.clone(),
            )),
            code,
            literals,
        );
    }
}

#[cfg(test)]
mod tests {
    use dcbor::prelude::*;

    use super::*;
    use bc_envelope::Envelope;

    #[test]
    fn test_array_pattern_any() {
        // Create a CBOR array directly
        let cbor_array = vec![1, 2, 3].to_cbor();
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
        let cbor_array = vec![1, 2, 3].to_cbor();
        let envelope = Envelope::new(cbor_array);

        // Test exact count
        let pattern = ArrayPattern::count(3);
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);

        // Test count range
        let pattern = ArrayPattern::range_count(2..=4);
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);

        // Test count mismatch
        let pattern = ArrayPattern::count(5);
        let paths = pattern.paths(&envelope);
        assert!(paths.is_empty());
    }
}
