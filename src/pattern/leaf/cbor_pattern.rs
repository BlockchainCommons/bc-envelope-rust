use dcbor::prelude::*;

use crate::{
    Envelope,
    pattern::{Matcher, Path},
};

/// Pattern for matching specific CBOR values.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum CborPattern {
    /// Matches any CBOR value.
    Any,
    /// Matches the specific CBOR value.
    Exact(CBOR),
}

impl CborPattern {
    /// Creates a new `CborPattern` that matches any CBOR value.
    pub fn any() -> Self { CborPattern::Any }

    /// Creates a new `CborPattern` that matches a specific CBOR value.
    pub fn exact(cbor: CBOR) -> Self { CborPattern::Exact(cbor) }
}

impl Matcher for CborPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        match self {
            CborPattern::Any => vec![vec![envelope.clone()]],
            CborPattern::Exact(expected_cbor) => {
                let subject_cbor = envelope.subject().to_cbor();
                if subject_cbor == *expected_cbor {
                    vec![vec![envelope.clone()]]
                } else {
                    vec![]
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Envelope;

    #[test]
    fn test_cbor_pattern_any() {
        let envelope = Envelope::new("test");
        let pattern = CborPattern::any();
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec![envelope.clone()]);
    }

    #[test]
    fn test_cbor_pattern_exact() {
        let value = "test_value";
        let envelope = Envelope::new(value);
        let cbor = envelope.subject().to_cbor(); // Use the same CBOR as the envelope
        let pattern = CborPattern::exact(cbor);
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec![envelope.clone()]);

        // Test with different value
        let different_envelope = Envelope::new("different");
        let paths = pattern.paths(&different_envelope);
        assert!(paths.is_empty());
    }
}
