use crate::Envelope;

use super::envelope::EnvelopeCase;

/// The type of incoming edge provided to the visitor.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum EdgeType {
    None,
    Subject,
    Assertion,
    Predicate,
    Object,
    Wrapped,
}

impl EdgeType {
    pub fn label(&self) -> Option<&'static str> {
        match self {
            EdgeType::Subject | EdgeType::Wrapped => Some("subj"),
            EdgeType::Predicate => Some("pred"),
            EdgeType::Object => Some("obj"),
            _ => None,
        }
    }
}

/// A visitor function that is called for each node in the envelope.
pub type Visitor<'a, Parent> = dyn Fn(Envelope, usize, EdgeType, Option<Parent>) -> Option<Parent> + 'a;

/// Functions for walking an envelope.
impl Envelope {
    /// Walk the envelope, calling the visitor function for each element.
    ///
    /// If `hide_nodes` is true, then the visitor function will not be called for nodes,
    /// but only for the children of nodes.
    pub fn walk<Parent: Clone>(&self, hide_nodes: bool, visit: &Visitor<'_, Parent>) {
        if hide_nodes {
            self.walk_tree(visit);
        } else {
            self.walk_structure(visit);
        }
    }

    fn walk_structure<Parent: Clone>(&self, visit: &Visitor<'_, Parent>) {
        self._walk_structure(0, EdgeType::None, None, visit);
    }

    fn _walk_structure<Parent: Clone>(&self, level: usize, incoming_edge: EdgeType, parent: Option<Parent>, visit: &Visitor<'_, Parent>) {
        let parent = visit(self.clone(), level, incoming_edge, parent);
        let next_level = level + 1;
        match self.case() {
            EnvelopeCase::Node { subject, assertions, .. } => {
                subject.clone()._walk_structure(next_level, EdgeType::Subject, parent.clone(), visit);
                for assertion in assertions {
                    assertion.clone()._walk_structure(next_level, EdgeType::Assertion, parent.clone(), visit);
                }
            },
            EnvelopeCase::Wrapped { envelope, .. } => {
                envelope.clone()._walk_structure(next_level, EdgeType::Wrapped, parent, visit);
            },
            EnvelopeCase::Assertion(assertion) => {
                assertion.predicate()._walk_structure(next_level, EdgeType::Predicate, parent.clone(), visit);
                assertion.object()._walk_structure(next_level, EdgeType::Object, parent, visit);
            },
            _ => {},
        }
    }

    fn walk_tree<Parent: Clone>(&self, visit: &Visitor<'_, Parent>)
    {
        self._walk_tree(0, None, visit);
    }

    fn _walk_tree<Parent: Clone>(&self, level: usize, parent: Option<Parent>, visit: &Visitor<'_, Parent>) -> Option<Parent> {
        let mut parent = parent;
        let mut subject_level = level;
        if !self.is_node() {
            parent = visit(self.clone(), level, EdgeType::None, parent);
            subject_level = level + 1;
        }
        match self.case() {
            EnvelopeCase::Node { subject, assertions, .. } => {
                let assertion_parent = subject.clone()._walk_tree(subject_level, parent.clone(), visit);
                let assertion_level = subject_level + 1;
                for assertion in assertions {
                    assertion.clone()._walk_tree(assertion_level, assertion_parent.clone(), visit);
                }
            },
            EnvelopeCase::Wrapped { envelope, .. } => {
                envelope.clone()._walk_tree(subject_level, parent.clone(), visit);
            },
            EnvelopeCase::Assertion(assertion) => {
                assertion.predicate()._walk_tree(subject_level, parent.clone(), visit);
                assertion.object()._walk_tree(subject_level, parent.clone(), visit);
            },
            _ => {},
        }
        parent
    }
}
