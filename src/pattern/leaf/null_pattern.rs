use crate::{
    Envelope,
    pattern::{Matcher, Path},
};

/// Pattern for matching null values.
#[derive(Debug, Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Envelope;

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
