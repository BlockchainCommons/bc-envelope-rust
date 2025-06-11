use super::{Matcher, Path, Pattern};
use crate::Envelope;

#[derive(Debug, Clone)]
pub enum PredicatePattern {
    Any,
    Pattern(Pattern),
}

impl PredicatePattern {
    pub fn any() -> Self { PredicatePattern::Any }

    pub fn pattern(pattern: Pattern) -> Self {
        PredicatePattern::Pattern(pattern)
    }
}

impl Matcher for PredicatePattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        if let Some(predicate) = envelope.as_predicate() {
            match self {
                PredicatePattern::Any => {
                    vec![vec![envelope.clone(), predicate.clone()]]
                }
                PredicatePattern::Pattern(pattern) => {
                    if pattern.matches(&predicate) {
                        vec![vec![envelope.clone(), predicate.clone()]]
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
