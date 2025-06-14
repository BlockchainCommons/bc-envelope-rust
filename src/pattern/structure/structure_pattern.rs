use super::{
    AssertionsPattern, DigestPattern, NodePattern, ObjectPattern,
    ObscuredPattern, PredicatePattern, SubjectPattern, WrappedPattern,
};
use crate::{
    Envelope,
    pattern::{Matcher, Path, Pattern, vm::Instr},
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

    pub fn compile(&self, code: &mut Vec<Instr>, lits: &mut Vec<Pattern>) {
        match self {
            StructurePattern::Subject(s) => s.compile(code, lits),
            // For structure patterns that have complex path logic, we need to
            // use a different approach than atomic matching
            StructurePattern::Assertions(_)
            | StructurePattern::Wrapped(_)
            | StructurePattern::Object(_) => {
                // Use MatchStructure instead of MatchPredicate for patterns
                // that return specific paths
                let idx = lits.len();
                lits.push(crate::pattern::Pattern::Structure(self.clone()));
                code.push(Instr::MatchStructure(idx));
            }
            // For other structure patterns, fall back to atomic matching for
            // now
            _ => {
                let idx = lits.len();
                lits.push(crate::pattern::Pattern::Structure(self.clone()));
                code.push(Instr::MatchPredicate(idx));
            }
        }
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
