use dcbor::prelude::*;

use crate::{
    Envelope,
    pattern::{Matcher, Path},
};

/// Pattern for matching tag values.
#[derive(Debug, Clone)]
pub enum TaggedPattern {
    /// Matches any tagged leaf.
    Any,
    /// Matches any leaf with the specific tag.
    Tag(Tag),
    /// Matches a leaf with a tag having the given name in the global tags
    /// registry.
    Named(String),
    /// Matches a leaf with a tag whose name matches the given regex pattern.
    Regex(regex::Regex),
}

impl TaggedPattern {
    /// Creates a new `TaggedPattern` that matches any tag.
    pub fn any() -> Self { TaggedPattern::Any }

    /// Creates a new `TaggedPattern` that matches a specific tag.
    pub fn with_tag(tag: Tag) -> Self { TaggedPattern::Tag(tag) }

    /// Creates a new `TaggedPattern` that matches a specific tag value.
    pub fn with_tag_value(value: u64) -> Self {
        TaggedPattern::Tag(Tag::with_value(value))
    }

    /// Creates a new `TaggedPattern` that matches a tag by its name in the
    /// global tags registry.
    pub fn with_tag_name(name: impl Into<String>) -> Self {
        TaggedPattern::Named(name.into())
    }

    /// Creates a new `TaggedPattern` that matches tags whose names match the
    /// given regex pattern.
    pub fn with_tag_regex(regex: regex::Regex) -> Self {
        TaggedPattern::Regex(regex)
    }
}

impl Matcher for TaggedPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        // Check if the envelope subject contains a tagged value
        if let Some(cbor) = envelope.subject().as_leaf() {
            if let CBORCase::Tagged(tag, _) = cbor.as_case() {
                match self {
                    TaggedPattern::Any => vec![vec![envelope.clone()]],
                    TaggedPattern::Tag(expected_tag) => {
                        if expected_tag.value() == tag.value() {
                            vec![vec![envelope.clone()]]
                        } else {
                            vec![]
                        }
                    }
                    TaggedPattern::Named(name) => {
                        // Look up the tag by name in the global tags registry
                        with_tags!(|tags: &TagsStore| {
                            if let Some(expected_tag) = tags.tag_for_name(name)
                            {
                                if expected_tag.value() == tag.value() {
                                    vec![vec![envelope.clone()]]
                                } else {
                                    vec![]
                                }
                            } else {
                                // Name not found in registry, no match
                                vec![]
                            }
                        })
                    }
                    TaggedPattern::Regex(regex) => {
                        // Check if the tag's name (from registry) matches the
                        // regex
                        with_tags!(|tags: &TagsStore| {
                            if let Some(tag_name) =
                                tags.assigned_name_for_tag(tag)
                            {
                                if regex.is_match(&tag_name) {
                                    vec![vec![envelope.clone()]]
                                } else {
                                    vec![]
                                }
                            } else {
                                // Tag has no name in registry, no match
                                vec![]
                            }
                        })
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

        let pattern = TaggedPattern::any();
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
        let pattern = TaggedPattern::with_tag_value(100);
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);

        // Test non-matching tag
        let pattern = TaggedPattern::with_tag_value(200);
        let paths = pattern.paths(&envelope);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_tag_pattern_named() {
        // Ensure tags are registered for testing
        dcbor::register_tags();
        bc_components::register_tags();

        // Create a tagged envelope using a registered tag (e.g., date tag = 1)
        let tagged_cbor = CBOR::to_tagged_value(1, "2023-12-25");
        let envelope = Envelope::new(tagged_cbor);

        // Test matching by name (dcbor registers tag 1 as "date")
        let pattern = TaggedPattern::with_tag_name("date");
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec![envelope.clone()]);

        // Test with non-matching name
        let pattern = TaggedPattern::with_tag_name("unknown_tag");
        let paths = pattern.paths(&envelope);
        assert!(paths.is_empty());

        // Test with non-tagged envelope
        let text_envelope = Envelope::new("test");
        let pattern = TaggedPattern::with_tag_name("date");
        let paths = pattern.paths(&text_envelope);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_tag_pattern_regex() {
        // Ensure tags are registered for testing
        dcbor::register_tags();
        bc_components::register_tags();

        // Create a tagged envelope using a registered tag (e.g., date tag = 1)
        let tagged_cbor = CBOR::to_tagged_value(1, "2023-12-25");
        let envelope = Envelope::new(tagged_cbor);

        // Test regex that matches "date"
        let regex = regex::Regex::new(r"^da.*").unwrap();
        let pattern = TaggedPattern::with_tag_regex(regex);
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec![envelope.clone()]);

        // Test regex that matches names ending with "te"
        let regex = regex::Regex::new(r".*te$").unwrap();
        let pattern = TaggedPattern::with_tag_regex(regex);
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec![envelope.clone()]);

        // Test regex that doesn't match
        let regex = regex::Regex::new(r"^unknown.*").unwrap();
        let pattern = TaggedPattern::with_tag_regex(regex);
        let paths = pattern.paths(&envelope);
        assert!(paths.is_empty());

        // Test with non-tagged envelope
        let text_envelope = Envelope::new("test");
        let regex = regex::Regex::new(r".*").unwrap();
        let pattern = TaggedPattern::with_tag_regex(regex);
        let paths = pattern.paths(&text_envelope);
        assert!(paths.is_empty());

        // Test with unregistered tag (should not match any regex)
        let unregistered_tagged_cbor =
            CBOR::to_tagged_value(999, "unregistered_value");
        let unregistered_envelope = Envelope::new(unregistered_tagged_cbor);
        let regex = regex::Regex::new(r".*").unwrap(); // Match everything
        let pattern = TaggedPattern::with_tag_regex(regex);
        let paths = pattern.paths(&unregistered_envelope);
        assert!(paths.is_empty()); // Should be empty because tag 999 has no name in registry
    }
}
