#[cfg(feature = "known_value")]
use known_values::KnownValue;

use crate::{
    Envelope,
    pattern::{Matcher, Path},
};

/// Pattern for matching known values.
#[derive(Debug, Clone)]
#[cfg(feature = "known_value")]
pub enum KnownValuePattern {
    /// Matches any known value.
    Any,
    /// Matches the specific known value.
    KnownValue(KnownValue),
}

#[cfg(feature = "known_value")]
impl KnownValuePattern {
    /// Creates a new `KnownValuePattern` that matches any known value.
    pub fn any() -> Self { KnownValuePattern::Any }

    /// Creates a new `KnownValuePattern` that matches a specific known value.
    pub fn known_value(value: KnownValue) -> Self {
        KnownValuePattern::KnownValue(value)
    }
}

#[cfg(feature = "known_value")]
impl Matcher for KnownValuePattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        if let Ok(value) = envelope.extract_subject::<KnownValue>() {
            match self {
                KnownValuePattern::Any => vec![vec![envelope.clone()]],
                KnownValuePattern::KnownValue(expected) => {
                    if value == *expected {
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

#[cfg(all(test, feature = "known_value"))]
mod tests {
    use super::*;
    use crate::Envelope;

    #[test]
    fn test_known_value_pattern_any() {
        let value = KnownValue::new(1);
        let envelope = Envelope::new(value.clone());
        let pattern = KnownValuePattern::any();
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec![envelope.clone()]);

        // Test with non-known-value envelope
        let text_envelope = Envelope::new("test");
        let paths = pattern.paths(&text_envelope);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_known_value_pattern_specific() {
        let value = KnownValue::new(1);
        let envelope = Envelope::new(value.clone());
        let pattern = KnownValuePattern::known_value(value.clone());
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec![envelope.clone()]);

        // Test with different known value
        let different_value = KnownValue::new(2);
        let pattern = KnownValuePattern::known_value(different_value);
        let paths = pattern.paths(&envelope);
        assert!(paths.is_empty());
    }
}
