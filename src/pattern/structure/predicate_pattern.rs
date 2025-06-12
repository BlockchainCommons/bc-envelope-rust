use crate::{
    Envelope,
    pattern::{Matcher, Path, Pattern},
};

#[derive(Debug, Clone)]
pub enum PredicatePattern {
    Any,
    Pattern(Box<Pattern>),
}

impl PredicatePattern {
    pub fn any() -> Self { PredicatePattern::Any }

    pub fn pattern(pattern: Pattern) -> Self {
        PredicatePattern::Pattern(Box::new(pattern))
    }
}

impl Matcher for PredicatePattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        if let Some(predicate) = envelope.as_predicate() {
            match self {
                PredicatePattern::Any => {
                    vec![vec![predicate.clone()]]
                }
                PredicatePattern::Pattern(pattern) => {
                    if pattern.matches(&predicate) {
                        vec![vec![predicate.clone()]]
                    } else {
                        vec![]
                    }
                }
            }
        } else {
            vec![]
        }
    }
}
