use super::{BoolPattern, Matcher, NumberPattern, Path, TextPattern};
use crate::Envelope;

/// Pattern for matching leaf values.
#[derive(Debug, Clone)]
pub enum LeafPattern {
    /// Matches any leaf.
    Any,
    /// Matches the specific CBOR.
    // CBOR(CBOR),
    /// Matches a numeric value.
    Number(NumberPattern),
    /// Matches a text value.
    Text(TextPattern),
    /// Matches a byte string value.
    // ByteString(ByteStringPattern),
    /// Matches a tag value.
    // Tag(TagPattern),
    /// Matches an array.
    // Array(ArrayPattern),
    /// Matches a map.
    // Map(MapPattern),
    /// Matches a boolean value.
    Boolean(BoolPattern),
    ///// Matches the null value.
    //// Null,
}

impl LeafPattern {
    /// Creates a new `LeafPattern` that matches any leaf.
    pub fn any() -> Self { LeafPattern::Any }

    pub fn number(pattern: NumberPattern) -> Self {
        LeafPattern::Number(pattern)
    }

    pub fn text(pattern: TextPattern) -> Self { LeafPattern::Text(pattern) }

    pub fn boolean(pattern: BoolPattern) -> Self {
        LeafPattern::Boolean(pattern)
    }
}

impl Matcher for LeafPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        match self {
            LeafPattern::Any => {
                if envelope.is_leaf() {
                    vec![vec![envelope.clone()]]
                } else {
                    vec![]
                }
            }
            LeafPattern::Number(pattern) => {
                pattern.paths(envelope)
            }
            LeafPattern::Text(pattern) => {
                pattern.paths(envelope)
            }
            LeafPattern::Boolean(pattern) => {
                pattern.paths(envelope)
            }
        }
    }
}
