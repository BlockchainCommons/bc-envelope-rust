use super::{
    AssertionsPattern, ObjectPattern, PredicatePattern, SubjectPattern,
    WrappedPattern,
};
use crate::{
    Envelope,
    pattern::{Matcher, Path},
};

/// Pattern for matching envelope structure elements.
#[derive(Debug, Clone)]
pub enum StructurePattern {
    /// Matches assertions.
    Assertions(AssertionsPattern),
    /// Matches objects.
    Object(ObjectPattern),
    /// Matches predicates.
    Predicate(PredicatePattern),
    /// Matches subjects.
    Subject(SubjectPattern),
    /// Matches wrapped envelopes.
    Wrapped(WrappedPattern),
    // Matches digests.
    // Digest(DigestPattern),
    // Matches nodes.
    // Node(NodePattern),
    // Matches obscured elements.
    // Obscured(ObscuredPattern),
}

impl StructurePattern {
    pub fn assertions(pattern: AssertionsPattern) -> Self {
        StructurePattern::Assertions(pattern)
    }

    pub fn object(pattern: ObjectPattern) -> Self {
        StructurePattern::Object(pattern)
    }

    pub fn predicate(pattern: PredicatePattern) -> Self {
        StructurePattern::Predicate(pattern)
    }

    pub fn subject(pattern: SubjectPattern) -> Self {
        StructurePattern::Subject(pattern)
    }

    pub fn wrapped(pattern: WrappedPattern) -> Self {
        StructurePattern::Wrapped(pattern)
    }
}

impl Matcher for StructurePattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        match self {
            StructurePattern::Assertions(pattern) => pattern.paths(envelope),
            StructurePattern::Object(pattern) => pattern.paths(envelope),
            StructurePattern::Predicate(pattern) => pattern.paths(envelope),
            StructurePattern::Subject(pattern) => pattern.paths(envelope),
            StructurePattern::Wrapped(pattern) => pattern.paths(envelope),
        }
    }
}
