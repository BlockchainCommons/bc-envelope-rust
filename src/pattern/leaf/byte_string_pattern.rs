use crate::{
    Envelope,
    pattern::{Matcher, Path},
};

/// Pattern for matching byte string values.
#[derive(Debug, Clone)]
pub enum ByteStringPattern {
    /// Matches any byte string.
    Any,
    /// Matches the specific byte string.
    Exact(Vec<u8>),
}

impl ByteStringPattern {
    /// Creates a new `ByteStringPattern` that matches any byte string.
    pub fn any() -> Self { ByteStringPattern::Any }

    /// Creates a new `ByteStringPattern` that matches a specific byte string.
    pub fn exact(value: Vec<u8>) -> Self { ByteStringPattern::Exact(value) }
}

impl Matcher for ByteStringPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        if let Some(bytes) = envelope.subject().as_byte_string() {
            match self {
                ByteStringPattern::Any => vec![vec![envelope.clone()]],
                ByteStringPattern::Exact(value) => {
                    if &bytes == value {
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
    fn test_byte_string_pattern_any() {
        let bytes = vec![1, 2, 3, 4];
        let envelope = Envelope::new(CBOR::to_byte_string(bytes));
        let pattern = ByteStringPattern::any();
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec![envelope.clone()]);

        // Test with non-byte-string envelope
        let text_envelope = Envelope::new("test");
        let paths = pattern.paths(&text_envelope);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_byte_string_pattern_exact() {
        let bytes = vec![1, 2, 3, 4];
        let envelope = Envelope::new(CBOR::to_byte_string(bytes.clone()));
        let pattern = ByteStringPattern::exact(bytes.clone());
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec![envelope.clone()]);

        // Test with different byte string
        let different_bytes = vec![5, 6, 7, 8];
        let pattern = ByteStringPattern::exact(different_bytes);
        let paths = pattern.paths(&envelope);
        assert!(paths.is_empty());
    }
}
