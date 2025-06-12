#[cfg(feature = "known_value")]
use known_values::KnownValue;

#[cfg(feature = "known_value")]
use super::leaf::KnownValuePattern;
use super::{
    Matcher, Path,
    leaf::{BoolPattern, LeafPattern, NumberPattern, TextPattern},
    meta::{
        AndPattern, MetaPattern, OrPattern, SearchPattern, SequencePattern,
    },
    structure::{
        AssertionsPattern, DigestPattern, NodePattern, ObjectPattern,
        ObscuredPattern, PredicatePattern, StructurePattern, SubjectPattern,
        WrappedPattern,
    },
};
use crate::Envelope;

/// The main pattern type used for matching envelopes.
#[derive(Debug, Clone)]
pub enum Pattern {
    /// Matches any element.
    Any,
    /// Never matches any element.
    None,

    /// Meta-patterns for combining and modifying other patterns.
    Meta(MetaPattern),

    /// Structure patterns for matching envelope elements.
    Structure(StructurePattern),

    /// Leaf patterns for matching CBOR values.
    Leaf(LeafPattern),
}

impl Pattern {
    /// Creates a new `Pattern` that matches any element.
    pub fn any() -> Self { Pattern::Any }

    /// Creates a new `Pattern` that never matches any element.
    pub fn none() -> Self { Pattern::None }
}

impl Pattern {
    pub fn any_bool() -> Self {
        Pattern::Leaf(LeafPattern::Boolean(BoolPattern::any()))
    }

    pub fn bool(b: bool) -> Self {
        Pattern::Leaf(LeafPattern::Boolean(BoolPattern::exact(b)))
    }
}

impl Pattern {
    pub fn any_text() -> Self {
        Pattern::Leaf(LeafPattern::Text(TextPattern::any()))
    }

    pub fn text<T: Into<String>>(value: T) -> Self {
        Pattern::Leaf(LeafPattern::Text(TextPattern::exact(value)))
    }

    pub fn text_regex(regex: &regex::Regex) -> Self {
        Pattern::Leaf(LeafPattern::Text(TextPattern::regex(regex.clone())))
    }
}

impl Pattern {
    pub fn any_number() -> Self {
        Pattern::Leaf(LeafPattern::Number(NumberPattern::any()))
    }

    pub fn number<T: Into<f64>>(value: T) -> Self {
        Pattern::Leaf(LeafPattern::Number(NumberPattern::exact(value)))
    }

    pub fn number_range<A: Into<f64> + Copy>(
        range: std::ops::RangeInclusive<A>,
    ) -> Self {
        Pattern::Leaf(LeafPattern::Number(NumberPattern::range(range)))
    }

    pub fn number_greater_than<T: Into<f64>>(value: T) -> Self {
        Pattern::Leaf(LeafPattern::Number(NumberPattern::greater_than(value)))
    }

    pub fn number_greater_than_or_equal<T: Into<f64>>(value: T) -> Self {
        Pattern::Leaf(LeafPattern::Number(
            NumberPattern::greater_than_or_equal(value),
        ))
    }

    pub fn number_less_than<T: Into<f64>>(value: T) -> Self {
        Pattern::Leaf(LeafPattern::Number(NumberPattern::less_than(value)))
    }

    pub fn number_less_than_or_equal<T: Into<f64>>(value: T) -> Self {
        Pattern::Leaf(LeafPattern::Number(NumberPattern::less_than_or_equal(
            value,
        )))
    }

    pub fn number_nan() -> Self {
        Pattern::Leaf(LeafPattern::Number(NumberPattern::nan()))
    }
}

#[cfg(feature = "known_value")]
impl Pattern {
    pub fn any_known_value() -> Self {
        Pattern::Leaf(LeafPattern::KnownValue(KnownValuePattern::any()))
    }

    pub fn known_value(value: KnownValue) -> Self {
        Pattern::Leaf(LeafPattern::KnownValue(KnownValuePattern::known_value(
            value,
        )))
    }

    pub fn known_value_named<T: Into<String>>(name: T) -> Self {
        Pattern::Leaf(LeafPattern::KnownValue(KnownValuePattern::named(name)))
    }

    pub fn known_value_regex(regex: regex::Regex) -> Self {
        Pattern::Leaf(LeafPattern::KnownValue(KnownValuePattern::regex(regex)))
    }
}

impl Pattern {
    pub fn and(patterns: Vec<Pattern>) -> Self {
        Pattern::Meta(MetaPattern::and(AndPattern::new(patterns)))
    }

    pub fn or(patterns: Vec<Pattern>) -> Self {
        Pattern::Meta(MetaPattern::or(OrPattern::new(patterns)))
    }
}

impl Pattern {
    pub fn any_wrapped() -> Self {
        Pattern::Structure(StructurePattern::wrapped(WrappedPattern::any()))
    }

    pub fn unwrap() -> Self {
        Pattern::Structure(StructurePattern::wrapped(WrappedPattern::unwrap()))
    }
}

impl Pattern {
    pub fn any_assertion() -> Self {
        Pattern::Structure(StructurePattern::assertions(
            AssertionsPattern::any(),
        ))
    }

    pub fn assertion_with_predicate(pattern: Pattern) -> Self {
        Pattern::Structure(StructurePattern::assertions(
            AssertionsPattern::with_predicate(pattern),
        ))
    }

    pub fn assertion_with_object(pattern: Pattern) -> Self {
        Pattern::Structure(StructurePattern::assertions(
            AssertionsPattern::with_object(pattern),
        ))
    }
}

impl Pattern {
    pub fn sequence(patterns: Vec<Pattern>) -> Self {
        Pattern::Meta(MetaPattern::sequence(SequencePattern::new(patterns)))
    }
}

impl Pattern {
    pub fn subject() -> Self {
        Pattern::Structure(StructurePattern::subject(SubjectPattern::any()))
    }

    pub fn any_predicate() -> Self {
        Pattern::Structure(StructurePattern::predicate(PredicatePattern::any()))
    }

    pub fn predicate(pattern: Pattern) -> Self {
        Pattern::Structure(StructurePattern::predicate(
            PredicatePattern::pattern(pattern),
        ))
    }

    pub fn any_object() -> Self {
        Pattern::Structure(StructurePattern::object(ObjectPattern::any()))
    }

    pub fn object(pattern: Pattern) -> Self {
        Pattern::Structure(StructurePattern::object(ObjectPattern::pattern(
            pattern,
        )))
    }
}

impl Pattern {
    pub fn digest(digest: bc_components::Digest) -> Self {
        Pattern::Structure(StructurePattern::digest(DigestPattern::digest(
            digest,
        )))
    }

    pub fn digest_hex_prefix<T: Into<String>>(prefix: T) -> Self {
        Pattern::Structure(StructurePattern::digest(DigestPattern::hex_prefix(
            prefix,
        )))
    }

    pub fn digest_binary_regex(regex: regex::bytes::Regex) -> Self {
        Pattern::Structure(StructurePattern::digest(
            DigestPattern::binary_regex(regex),
        ))
    }

    pub fn any_node() -> Self {
        Pattern::Structure(StructurePattern::node(NodePattern::any()))
    }

    pub fn node_with_assertions_count_range(
        range: std::ops::RangeInclusive<usize>,
    ) -> Self {
        Pattern::Structure(StructurePattern::node(
            NodePattern::assertions_count_range(range),
        ))
    }

    pub fn node_with_assertions_count(count: usize) -> Self {
        Pattern::Structure(StructurePattern::node(
            NodePattern::assertions_count(count),
        ))
    }

    pub fn obscured() -> Self {
        Pattern::Structure(StructurePattern::obscured(ObscuredPattern::any()))
    }

    pub fn elided() -> Self {
        Pattern::Structure(
            StructurePattern::obscured(ObscuredPattern::elided()),
        )
    }

    pub fn encrypted() -> Self {
        Pattern::Structure(StructurePattern::obscured(
            ObscuredPattern::encrypted(),
        ))
    }

    pub fn compressed() -> Self {
        Pattern::Structure(StructurePattern::obscured(
            ObscuredPattern::compressed(),
        ))
    }
}

impl Pattern {
    pub fn search(pattern: Pattern) -> Self {
        Pattern::Meta(MetaPattern::search(SearchPattern::new(pattern)))
    }
}

impl Matcher for Pattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        match self {
            Pattern::Any => vec![vec![envelope.clone()]],
            Pattern::None => vec![],
            Pattern::Meta(pattern) => pattern.paths(envelope),
            Pattern::Structure(pattern) => pattern.paths(envelope),
            Pattern::Leaf(pattern) => pattern.paths(envelope),
        }
    }
}
