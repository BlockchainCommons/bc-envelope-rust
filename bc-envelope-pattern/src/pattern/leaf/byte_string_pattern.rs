use crate::{
    pattern::{compile_as_atomic, leaf::LeafPattern, vm::Instr, Compilable, Matcher, Path}, Envelope, Pattern
};

/// Pattern for matching byte string values.
#[derive(Debug, Clone)]
pub enum ByteStringPattern {
    /// Matches any byte string.
    Any,
    /// Matches the specific byte string.
    Exact(Vec<u8>),
    /// Matches the binary regular expression for a byte string.
    BinaryRegex(regex::bytes::Regex),
}

impl PartialEq for ByteStringPattern {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ByteStringPattern::Any, ByteStringPattern::Any) => true,
            (ByteStringPattern::Exact(a), ByteStringPattern::Exact(b)) => {
                a == b
            }
            (
                ByteStringPattern::BinaryRegex(a),
                ByteStringPattern::BinaryRegex(b),
            ) => a.as_str() == b.as_str(),
            _ => false,
        }
    }
}

impl Eq for ByteStringPattern {}

impl std::hash::Hash for ByteStringPattern {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            ByteStringPattern::Any => {
                0u8.hash(state);
            }
            ByteStringPattern::Exact(s) => {
                1u8.hash(state);
                s.hash(state);
            }
            ByteStringPattern::BinaryRegex(regex) => {
                2u8.hash(state);
                // Regex does not implement Hash, so we hash its pattern string.
                regex.as_str().hash(state);
            }
        }
    }
}

impl ByteStringPattern {
    /// Creates a new `ByteStringPattern` that matches any byte string.
    pub fn any() -> Self { ByteStringPattern::Any }

    /// Creates a new `ByteStringPattern` that matches a specific byte string.
    pub fn exact(value: impl AsRef<[u8]>) -> Self {
        ByteStringPattern::Exact(value.as_ref().to_vec())
    }

    /// Creates a new `ByteStringPattern` that matches the binary regex for a
    /// byte string.
    pub fn binary_regex(regex: regex::bytes::Regex) -> Self {
        ByteStringPattern::BinaryRegex(regex)
    }
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
                ByteStringPattern::BinaryRegex(regex) => {
                    if regex.is_match(&bytes) {
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

impl Compilable for ByteStringPattern {
    fn compile(&self, code: &mut Vec<Instr>, literals: &mut Vec<Pattern>) {
        compile_as_atomic(
            &Pattern::Leaf(LeafPattern::ByteString(
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

    #[test]
    fn test_byte_string_pattern_binary_regex() {
        let bytes = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]; // "Hello"
        let envelope = Envelope::new(CBOR::to_byte_string(bytes.clone()));

        // Test matching regex
        let regex = regex::bytes::Regex::new(r"^He.*o$").unwrap();
        let pattern = ByteStringPattern::binary_regex(regex);
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec![envelope.clone()]);

        // Test non-matching regex
        let regex = regex::bytes::Regex::new(r"^World").unwrap();
        let pattern = ByteStringPattern::binary_regex(regex);
        let paths = pattern.paths(&envelope);
        assert!(paths.is_empty());

        // Test with non-byte-string envelope
        let text_envelope = Envelope::new("test");
        let regex = regex::bytes::Regex::new(r".*").unwrap();
        let pattern = ByteStringPattern::binary_regex(regex);
        let paths = pattern.paths(&text_envelope);
        assert!(paths.is_empty());
    }
}
