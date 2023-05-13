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

pub type Visitor<'a, 'p, Parent> = dyn Fn(&'a Envelope, usize, EdgeType, Option<&'p Parent>) -> Option<&'p Parent>;
pub type TreeVisitor<'a, 'p, 'v, Parent> = dyn Fn(&'a Envelope, usize, Option<&'p Parent>) -> Option<&'p Parent> + 'v;

impl Envelope {
    pub fn walk<'a, 'p, 'v, Parent>(&'a self, hide_nodes: bool, visit: &'p Visitor<'a, 'p, Parent>) {
        if hide_nodes {
            // Convert `visit` into a `TreeVisitor`:
            let tree_visit = |envelope, level, parent| -> Option<&'p Parent> {
                visit(envelope, level, EdgeType::None, parent)
            };
            self.walk_tree(&tree_visit); // ERROR HERE
        } else {
            self.walk_structure(visit);
        }
    }

    fn walk_structure<'a, 'p, Parent>(&'a self, visit: &'p Visitor<'a, 'p, Parent>) {
        self._walk_structure(0, EdgeType::None, None, visit);
    }

    fn _walk_structure<'a, 'p, Parent>(&'a self, level: usize, incoming_edge: EdgeType, parent: Option<&'p Parent>, visit: &'p Visitor<'a, 'p, Parent>) {
        let parent = visit(self, level, incoming_edge, parent);
        let next_level = level + 1;
        match self {
            Envelope::Node { subject, assertions, .. } => {
                subject._walk_structure(next_level, EdgeType::Subject, parent, visit);
                for assertion in assertions {
                    assertion._walk_structure(next_level, EdgeType::Assertion, parent, visit);
                }
            },
            Envelope::Wrapped { envelope, .. } => {
                envelope._walk_structure(next_level, EdgeType::Wrapped, parent, visit);
            },
            Envelope::Assertion(assertion) => {
                assertion.predicate()._walk_structure(next_level, EdgeType::Predicate, parent, visit);
                assertion.object()._walk_structure(next_level, EdgeType::Object, parent, visit);
            },
            _ => {},
        }
    }

    fn walk_tree<'a, 'p, 'v, Parent>(&'a self, tree_visit: &'v TreeVisitor<'a, 'p, 'v, Parent>) {
        self._walk_tree(0, None, tree_visit);
    }

    fn _walk_tree<'a, 'p, 'v, Parent>(&'a self, level: usize, parent: Option<&'p Parent>, tree_visit: &'v TreeVisitor<'a, 'p, 'v, Parent>) {
        let parent = tree_visit(self, level, parent);
        let next_level = level + 1;
        match self {
            Envelope::Node { subject, assertions, .. } => {
                subject._walk_tree(next_level, parent, tree_visit);
                for assertion in assertions {
                    assertion._walk_tree(next_level, parent, tree_visit);
                }
            },
            Envelope::Wrapped { envelope, .. } => {
                envelope._walk_tree(next_level, parent, tree_visit);
            },
            Envelope::Assertion(assertion) => {
                assertion.predicate()._walk_tree(next_level, parent, tree_visit);
                assertion.object()._walk_tree(next_level, parent, tree_visit);
            },
            _ => {},
        }
    }
}
