use dcbor::prelude::*;

use crate::{
    Envelope,
    pattern::{Matcher, Path},
};

/// Pattern for matching tag values.
#[derive(Debug, Clone)]
pub enum TagPattern {
    /// Matches any tag.
    Any,
    /// Matches the specific tag.
    Tag(Tag),
}

impl TagPattern {
    /// Creates a new `TagPattern` that matches any tag.
    pub fn any() -> Self { TagPattern::Any }

    /// Creates a new `TagPattern` that matches a specific tag.
    pub fn tag(tag: Tag) -> Self { TagPattern::Tag(tag) }

    /// Creates a new `TagPattern` that matches a specific tag value.
    pub fn tag_value(value: u64) -> Self {
        TagPattern::Tag(Tag::with_value(value))
    }
}

impl Matcher for TagPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        // Check if the envelope subject contains a tagged value
        if let Some(cbor) = envelope.subject().as_leaf() {
            if let CBORCase::Tagged(tag, _) = cbor.as_case() {
                match self {
                    TagPattern::Any => vec![vec![envelope.clone()]],
                    TagPattern::Tag(expected_tag) => {
                        if expected_tag.value() == tag.value() {
                            vec![vec![envelope.clone()]]
                        } else {
                            vec![]
                        }
                    }
                }
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Envelope;

    #[test]
    fn test_tag_pattern_any() {
        // Create a tagged envelope
        let tagged_cbor = CBOR::to_tagged_value(100, "tagged_value");
        let envelope = Envelope::new(tagged_cbor);

        let pattern = TagPattern::any();
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec![envelope.clone()]);

        // Test with non-tagged envelope
        let text_envelope = Envelope::new("test");
        let paths = pattern.paths(&text_envelope);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_tag_pattern_tag() {
        // Create a tagged envelope
        let tagged_cbor = CBOR::to_tagged_value(100, "tagged_value");
        let envelope = Envelope::new(tagged_cbor);

        // Test matching tag
        let pattern = TagPattern::tag_value(100);
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);

        // Test non-matching tag
        let pattern = TagPattern::tag_value(200);
        let paths = pattern.paths(&envelope);
        assert!(paths.is_empty());
    }
}
