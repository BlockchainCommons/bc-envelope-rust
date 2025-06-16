use crate::{
    Envelope, Pattern,
    pattern::{
        Compilable, Matcher, Path, compile_as_atomic, leaf::LeafPattern,
        vm::Instr,
    },
};

/// Pattern for matching null values.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum NullPattern {
    /// Matches any null value.
    Any,
}

impl NullPattern {
    /// Creates a new `NullPattern` that matches any null value.
    pub fn any() -> Self { NullPattern::Any }
}

impl Matcher for NullPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        match self {
            NullPattern::Any => {
                if envelope.subject().is_null() {
                    vec![vec![envelope.clone()]]
                } else {
                    vec![]
                }
            }
        }
    }
}

impl Compilable for NullPattern {
    fn compile(&self, code: &mut Vec<Instr>, literals: &mut Vec<Pattern>) {
        compile_as_atomic(
            &Pattern::Leaf(LeafPattern::Null(self.clone())),
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
    fn test_null_pattern_any() {
        let null_envelope = Envelope::null();
        let pattern = NullPattern::any();
        let paths = pattern.paths(&null_envelope);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec![null_envelope.clone()]);

        // Test with non-null envelope
        let text_envelope = Envelope::new("test");
        let paths = pattern.paths(&text_envelope);
        assert!(paths.is_empty());
    }
}
