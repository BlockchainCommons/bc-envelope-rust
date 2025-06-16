use dcbor::prelude::*;

use crate::{
    Envelope, Pattern,
    pattern::{
        Compilable, Matcher, Path, compile_as_atomic, leaf::LeafPattern,
        vm::Instr,
    },
};

/// Pattern for matching specific CBOR values.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum CBORPattern {
    /// Matches any CBOR value.
    Any,
    /// Matches the specific CBOR value.
    Exact(CBOR),
}

impl CBORPattern {
    /// Creates a new `CborPattern` that matches any CBOR value.
    pub fn any() -> Self { CBORPattern::Any }

    /// Creates a new `CborPattern` that matches a specific CBOR value.
    pub fn exact(cbor: CBOR) -> Self { CBORPattern::Exact(cbor) }
}

impl Matcher for CBORPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        let subject = envelope.subject();
        let subject_cbor = match subject.as_leaf() {
            Some(cbor) => cbor,
            None => return vec![],
        };
        match self {
            CBORPattern::Any => vec![vec![envelope.clone()]],
            CBORPattern::Exact(expected_cbor) => {
                if subject_cbor == *expected_cbor {
                    vec![vec![envelope.clone()]]
                } else {
                    vec![]
                }
            }
        }
    }
}

impl Compilable for CBORPattern {
    fn compile(&self, code: &mut Vec<Instr>, literals: &mut Vec<Pattern>) {
        compile_as_atomic(
            &Pattern::Leaf(LeafPattern::Cbor(self.clone())),
            code,
            literals,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bc_envelope::Envelope;

    #[test]
    fn test_cbor_pattern_any() {
        let envelope = Envelope::new("test");
        let pattern = CBORPattern::any();
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec![envelope.clone()]);
    }

    #[test]
    fn test_cbor_pattern_exact() {
        let value = "test_value";
        let envelope = Envelope::new(value);
        let cbor = envelope.subject().as_leaf().unwrap().clone();
        let pattern = CBORPattern::exact(cbor);
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec![envelope.clone()]);

        // Test with different value
        let different_envelope = Envelope::new("different");
        let paths = pattern.paths(&different_envelope);
        assert!(paths.is_empty());
    }
}
