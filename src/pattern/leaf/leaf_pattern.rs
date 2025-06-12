#[cfg(feature = "known_value")]
use super::KnownValuePattern;
use super::{
    ArrayPattern, BoolPattern, ByteStringPattern, CborPattern, DatePattern, MapPattern,
    NullPattern, NumberPattern, TagPattern, TextPattern,
};
use crate::{
    Envelope,
    pattern::{Matcher, Path},
};

/// Pattern for matching leaf values.
#[derive(Debug, Clone)]
pub enum LeafPattern {
    /// Matches any leaf.
    Any,
    /// Matches the specific CBOR.
    Cbor(CborPattern),
    /// Matches a numeric value.
    Number(NumberPattern),
    /// Matches a text value.
    Text(TextPattern),
    /// Matches a byte string value.
    ByteString(ByteStringPattern),
    /// Matches a tag value.
    Tag(TagPattern),
    /// Matches an array.
    Array(ArrayPattern),
    /// Matches a map.
    Map(MapPattern),
    /// Matches a boolean value.
    Boolean(BoolPattern),
    /// Matches the null value.
    Null(NullPattern),
    /// Matches a date value.
    Date(DatePattern),
    /// Matches a known value.
    #[cfg(feature = "known_value")]
    KnownValue(KnownValuePattern),
}

impl LeafPattern {
    /// Creates a new `LeafPattern` that matches any leaf.
    pub fn any() -> Self { LeafPattern::Any }

    pub fn cbor(pattern: CborPattern) -> Self { LeafPattern::Cbor(pattern) }

    pub fn number(pattern: NumberPattern) -> Self {
        LeafPattern::Number(pattern)
    }

    pub fn text(pattern: TextPattern) -> Self { LeafPattern::Text(pattern) }

    pub fn byte_string(pattern: ByteStringPattern) -> Self {
        LeafPattern::ByteString(pattern)
    }

    pub fn tag(pattern: TagPattern) -> Self { LeafPattern::Tag(pattern) }

    pub fn array(pattern: ArrayPattern) -> Self { LeafPattern::Array(pattern) }

    pub fn map(pattern: MapPattern) -> Self { LeafPattern::Map(pattern) }

    pub fn boolean(pattern: BoolPattern) -> Self {
        LeafPattern::Boolean(pattern)
    }

    pub fn null(pattern: NullPattern) -> Self { LeafPattern::Null(pattern) }

    pub fn date(pattern: DatePattern) -> Self { LeafPattern::Date(pattern) }

    #[cfg(feature = "known_value")]
    pub fn known_value(pattern: KnownValuePattern) -> Self {
        LeafPattern::KnownValue(pattern)
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
            LeafPattern::Cbor(pattern) => pattern.paths(envelope),
            LeafPattern::Number(pattern) => pattern.paths(envelope),
            LeafPattern::Text(pattern) => pattern.paths(envelope),
            LeafPattern::ByteString(pattern) => pattern.paths(envelope),
            LeafPattern::Tag(pattern) => pattern.paths(envelope),
            LeafPattern::Array(pattern) => pattern.paths(envelope),
            LeafPattern::Map(pattern) => pattern.paths(envelope),
            LeafPattern::Boolean(pattern) => pattern.paths(envelope),
            LeafPattern::Null(pattern) => pattern.paths(envelope),
            LeafPattern::Date(pattern) => pattern.paths(envelope),
            #[cfg(feature = "known_value")]
            LeafPattern::KnownValue(pattern) => pattern.paths(envelope),
        }
    }
}
