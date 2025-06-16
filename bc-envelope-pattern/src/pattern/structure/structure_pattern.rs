use super::{
    AssertionsPattern, DigestPattern, NodePattern, ObjectPattern,
    ObscuredPattern, PredicatePattern, SubjectPattern, WrappedPattern,
};
use crate::{
    Envelope,
    pattern::{Compilable, Matcher, Path, Pattern, vm::Instr},
};

/// Pattern for matching envelope structure elements.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum StructurePattern {
    /// Matches assertions.
    Assertions(AssertionsPattern),
    /// Matches digests.
    Digest(DigestPattern),
    /// Matches nodes.
    Node(NodePattern),
    /// Matches objects.
    Object(ObjectPattern),
    /// Matches obscured elements.
    Obscured(ObscuredPattern),
    /// Matches predicates.
    Predicate(PredicatePattern),
    /// Matches subjects.
    Subject(SubjectPattern),
    /// Matches wrapped envelopes.
    Wrapped(WrappedPattern),
}

impl StructurePattern {
    pub fn assertions(pattern: AssertionsPattern) -> Self {
        StructurePattern::Assertions(pattern)
    }

    pub fn digest(pattern: DigestPattern) -> Self {
        StructurePattern::Digest(pattern)
    }

    pub fn node(pattern: NodePattern) -> Self {
        StructurePattern::Node(pattern)
    }

    pub fn object(pattern: ObjectPattern) -> Self {
        StructurePattern::Object(pattern)
    }

    pub fn obscured(pattern: ObscuredPattern) -> Self {
        StructurePattern::Obscured(pattern)
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
            StructurePattern::Digest(pattern) => pattern.paths(envelope),
            StructurePattern::Node(pattern) => pattern.paths(envelope),
            StructurePattern::Object(pattern) => pattern.paths(envelope),
            StructurePattern::Obscured(pattern) => pattern.paths(envelope),
            StructurePattern::Predicate(pattern) => pattern.paths(envelope),
            StructurePattern::Subject(pattern) => pattern.paths(envelope),
            StructurePattern::Wrapped(pattern) => pattern.paths(envelope),
        }
    }
}

impl Compilable for StructurePattern {
    fn compile(&self, code: &mut Vec<Instr>, lits: &mut Vec<Pattern>) {
        match self {
            StructurePattern::Subject(s) => s.compile(code, lits),
            StructurePattern::Assertions(s) => s.compile(code, lits),
            StructurePattern::Wrapped(s) => s.compile(code, lits),
            StructurePattern::Object(s) => s.compile(code, lits),
            StructurePattern::Digest(s) => s.compile(code, lits),
            StructurePattern::Node(s) => s.compile(code, lits),
            StructurePattern::Obscured(s) => s.compile(code, lits),
            StructurePattern::Predicate(s) => s.compile(code, lits),
        }
    }
}
