use super::{Matcher, Path, Pattern};
use crate::Envelope;

#[derive(Debug, Clone)]
pub enum PredicatePattern {
    Any,
    Pattern(Pattern),
}

impl PredicatePattern {
    pub fn any() -> Self {
        PredicatePattern::Any
    }

    pub fn pattern(pattern: Pattern) -> Self {
        PredicatePattern::Pattern(pattern)
    }
}

impl Matcher for PredicatePattern {
    fn paths(&self, envelope: &Envelope) -> impl Iterator<Item = Path> {
        if let Some(predicate) = envelope.as_predicate() {
            match self {
                PredicatePattern::Any => {
                    Some(Vec::from_iter([envelope.clone(), predicate.clone()])).into_iter()
                }
                PredicatePattern::Pattern(pattern) => {
                    if pattern.matches(&predicate) {
                        Some(Vec::from_iter([envelope.clone(), predicate.clone()])).into_iter()
                    } else {
                        None.into_iter()
                    }
                }
            }
        } else {
            return None.into_iter();
        }
    }
}
