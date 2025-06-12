#[cfg(feature = "known_value")]
use known_values::{KNOWN_VALUES, KnownValue};

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
    /// Matches the name of a known value.
    Named(String),
}

#[cfg(feature = "known_value")]
impl KnownValuePattern {
    /// Creates a new `KnownValuePattern` that matches any known value.
    pub fn any() -> Self { KnownValuePattern::Any }

    /// Creates a new `KnownValuePattern` that matches a specific known value.
    pub fn known_value(value: KnownValue) -> Self {
        KnownValuePattern::KnownValue(value)
    }

    /// Creates a new `KnownValuePattern` that matches a known value by name.
    pub fn named(name: impl Into<String>) -> Self {
        KnownValuePattern::Named(name.into())
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
                KnownValuePattern::Named(name) => {
                    // Look up the known value by name in the global registry
                    let binding = KNOWN_VALUES.get();
                    if let Some(known_values_store) = binding.as_ref() {
                        if let Some(expected_value) =
                            known_values_store.known_value_named(name)
                        {
                            if value == *expected_value {
                                vec![vec![envelope.clone()]]
                            } else {
                                vec![]
                            }
                        } else {
                            // Name not found in registry, no match
                            vec![]
                        }
                    } else {
                        // Registry not initialized, no match
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
        let value = known_values::DATE;
        let envelope = Envelope::new(value.clone());
        let pattern = KnownValuePattern::known_value(value.clone());
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec![envelope.clone()]);

        // Test with different known value
        let different_value = known_values::LANGUAGE;
        let pattern = KnownValuePattern::known_value(different_value);
        let paths = pattern.paths(&envelope);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_known_value_pattern_named() {
        let value = known_values::DATE;
        let envelope = Envelope::new(value.clone());

        // Test matching by name
        let pattern = KnownValuePattern::named("date");
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec![envelope.clone()]);

        // Test with non-matching name
        let pattern = KnownValuePattern::named("language");
        let paths = pattern.paths(&envelope);
        assert!(paths.is_empty());

        // Test with unknown name
        let pattern = KnownValuePattern::named("unknown_name");
        let paths = pattern.paths(&envelope);
        assert!(paths.is_empty());

        // Test with non-known-value envelope
        let text_envelope = Envelope::new("test");
        let pattern = KnownValuePattern::named("date");
        let paths = pattern.paths(&text_envelope);
        assert!(paths.is_empty());
    }
}
