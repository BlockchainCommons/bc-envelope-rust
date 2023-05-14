use crate::Envelope;

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

pub type Visitor<'a, Parent> = dyn Fn(&'a Envelope, usize, EdgeType, Parent) -> Parent + 'a;

impl Envelope {
    pub fn walk<'a, Parent: Default + Clone>(&'a self, hide_nodes: bool, visit: &'a Visitor<'a, Parent>) {
        if hide_nodes {
            self.walk_tree(visit);
        } else {
            self.walk_structure(visit);
        }
    }

    fn walk_structure<'a, Parent: Default + Clone>(&'a self, visit: &'a Visitor<'a, Parent>) {
        self._walk_structure(0, EdgeType::None, Default::default(), visit);
    }

    fn _walk_structure<'a, Parent: Clone>(&'a self, level: usize, incoming_edge: EdgeType, parent: Parent, visit: &'a Visitor<'a, Parent>) {
        let parent = visit(self, level, incoming_edge, parent);
        let next_level = level + 1;
        match self {
            Envelope::Node { subject, assertions, .. } => {
                subject._walk_structure(next_level, EdgeType::Subject, parent.clone(), visit);
                for assertion in assertions {
                    assertion._walk_structure(next_level, EdgeType::Assertion, parent.clone(), visit);
                }
            },
            Envelope::Wrapped { envelope, .. } => {
                envelope._walk_structure(next_level, EdgeType::Wrapped, parent, visit);
            },
            Envelope::Assertion(assertion) => {
                assertion.predicate()._walk_structure(next_level, EdgeType::Predicate, parent.clone(), visit);
                assertion.object()._walk_structure(next_level, EdgeType::Object, parent, visit);
            },
            _ => {},
        }
    }

    fn walk_tree<'a, Parent: Default + Clone>(&'a self, visit: &'a Visitor<'a, Parent>)
    {
        self._walk_tree(0, Default::default(), visit);
    }

    fn _walk_tree<'a, Parent: Clone>(&'a self, level: usize, parent: Parent, visit: &'a Visitor<'a, Parent>) {
        let parent = visit(self, level, EdgeType::None, parent);
        let next_level = level + 1;
        match self {
            Envelope::Node { subject, assertions, .. } => {
                subject._walk_tree(next_level, parent.clone(), visit);
                for assertion in assertions {
                    assertion._walk_tree(next_level, parent.clone(), visit);
                }
            },
            Envelope::Wrapped { envelope, .. } => {
                envelope._walk_tree(next_level, parent, visit);
            },
            Envelope::Assertion(assertion) => {
                assertion.predicate()._walk_tree(next_level, parent.clone(), visit);
                assertion.object()._walk_tree(next_level, parent, visit);
            },
            _ => {},
        }
    }
}
