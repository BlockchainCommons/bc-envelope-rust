use super::{
    AndPattern, AssertionsPattern, LeafPattern, Matcher, OrPattern, Path,
    WrappedPattern,
};
use crate::{
    Envelope,
    pattern::{
        ObjectPattern, PredicatePattern, SequencePattern, SubjectPattern,
    },
};

/// The main pattern type used for matching envelopes.
#[derive(Debug, Clone)]
pub enum Pattern {
    /// Matches any element.
    Any,
    /// Never matches any element.
    None,
    /// Matches elements with a specific digest.
    // Digest(DigestPattern),
    /// Matches a node with specific properties.
    // Node(NodePattern),
    /// Matches a leaf with specific properties.
    Leaf(LeafPattern),
    /// Matches a known value.
    // KnownValue(KnownValuePattern),
    /// Matches a wrapped element.
    Wrapped(WrappedPattern),
    /// Matches an assertion element.
    // Assertion,
    /// Matches an obscured element.
    // Obscured(ObscuredPattern),
    /// Matches an element that matches all conditions.
    And(AndPattern),
    /// Matches an element that matches any condition.
    Or(OrPattern),
    /// Matches a sequence of elements.
    Assertion(AssertionsPattern),
    /// Matches a sequence of elements.
    Sequence(SequencePattern),
    Subject(SubjectPattern),
    Predicate(PredicatePattern),
    Object(ObjectPattern),
    ///// Matches an element with a specific cardinality.
    //// Repeat(RepeatPattern),
}

impl Pattern {
    /// Creates a new `Pattern` that matches any element.
    pub fn any() -> Self { Pattern::Any }

    /// Creates a new `Pattern` that never matches any element.
    pub fn none() -> Self { Pattern::None }
}

impl Pattern {
    pub fn any_bool() -> Self {
        Pattern::Leaf(LeafPattern::Boolean(super::BoolPattern::any()))
    }

    pub fn bool(b: bool) -> Self {
        Pattern::Leaf(LeafPattern::Boolean(super::BoolPattern::exact(b)))
    }
}

impl Pattern {
    pub fn any_text() -> Self {
        Pattern::Leaf(LeafPattern::Text(super::TextPattern::any()))
    }

    pub fn text<T: Into<String>>(value: T) -> Self {
        Pattern::Leaf(LeafPattern::Text(super::TextPattern::exact(value)))
    }

    pub fn text_regex(regex: &regex::Regex) -> Self {
        Pattern::Leaf(LeafPattern::Text(super::TextPattern::regex(
            regex.clone(),
        )))
    }
}

impl Pattern {
    pub fn any_number() -> Self {
        Pattern::Leaf(LeafPattern::Number(super::NumberPattern::any()))
    }

    pub fn number<T: Into<f64>>(value: T) -> Self {
        Pattern::Leaf(LeafPattern::Number(super::NumberPattern::exact(value)))
    }

    pub fn number_range<A: Into<f64> + Copy>(
        range: std::ops::RangeInclusive<A>,
    ) -> Self {
        Pattern::Leaf(LeafPattern::Number(super::NumberPattern::range(range)))
    }

    pub fn number_greater_than<T: Into<f64>>(value: T) -> Self {
        Pattern::Leaf(LeafPattern::Number(super::NumberPattern::greater_than(
            value,
        )))
    }

    pub fn number_greater_than_or_equal<T: Into<f64>>(value: T) -> Self {
        Pattern::Leaf(LeafPattern::Number(
            super::NumberPattern::greater_than_or_equal(value),
        ))
    }

    pub fn number_less_than<T: Into<f64>>(value: T) -> Self {
        Pattern::Leaf(LeafPattern::Number(super::NumberPattern::less_than(
            value,
        )))
    }

    pub fn number_less_than_or_equal<T: Into<f64>>(value: T) -> Self {
        Pattern::Leaf(LeafPattern::Number(
            super::NumberPattern::less_than_or_equal(value),
        ))
    }

    pub fn number_nan() -> Self {
        Pattern::Leaf(LeafPattern::Number(super::NumberPattern::nan()))
    }
}

impl Pattern {
    pub fn and(patterns: Vec<Pattern>) -> Self {
        Pattern::And(AndPattern::new(patterns))
    }

    pub fn or(patterns: Vec<Pattern>) -> Self {
        Pattern::Or(OrPattern::new(patterns))
    }
}

impl Pattern {
    pub fn any_wrapped() -> Self { Pattern::Wrapped(WrappedPattern::any()) }

    pub fn unwrap() -> Self { Pattern::Wrapped(WrappedPattern::unwrap()) }
}

impl Pattern {
    pub fn any_assertion() -> Self {
        Pattern::Assertion(AssertionsPattern::any())
    }

    pub fn assertion_with_predicate(pattern: Pattern) -> Self {
        Pattern::Assertion(AssertionsPattern::with_predicate(pattern))
    }

    pub fn assertion_with_object(pattern: Pattern) -> Self {
        Pattern::Assertion(AssertionsPattern::with_object(pattern))
    }
}

impl Pattern {
    pub fn sequence(patterns: Vec<Pattern>) -> Self {
        Pattern::Sequence(SequencePattern::new(patterns))
    }
}

impl Pattern {
    pub fn subject() -> Self { Pattern::Subject(SubjectPattern::any()) }

    pub fn any_predicate() -> Self {
        Pattern::Predicate(PredicatePattern::any())
    }

    pub fn predicate(pattern: Pattern) -> Self {
        Pattern::Predicate(PredicatePattern::pattern(pattern))
    }

    pub fn any_object() -> Self { Pattern::Object(ObjectPattern::any()) }

    pub fn object(pattern: Pattern) -> Self {
        Pattern::Object(ObjectPattern::pattern(pattern))
    }
}

impl Matcher for Pattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        match self {
            Pattern::Any => vec![vec![envelope.clone()]],
            Pattern::None => vec![],
            Pattern::Leaf(pattern) => pattern.paths(envelope),
            Pattern::And(pattern) => pattern.paths(envelope),
            Pattern::Or(pattern) => pattern.paths(envelope),
            Pattern::Wrapped(pattern) => pattern.paths(envelope),
            Pattern::Assertion(pattern) => pattern.paths(envelope),
            Pattern::Sequence(pattern) => pattern.paths(envelope),
            Pattern::Subject(pattern) => pattern.paths(envelope),
            Pattern::Predicate(pattern) => pattern.paths(envelope),
            Pattern::Object(pattern) => pattern.paths(envelope),
        }
    }
}
