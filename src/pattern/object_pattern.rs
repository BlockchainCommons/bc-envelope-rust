use super::{Matcher, Path, Pattern};
use crate::Envelope;

#[derive(Debug, Clone)]
pub enum ObjectPattern {
    Any,
    Pattern(Pattern),
}

impl ObjectPattern {
    pub fn any() -> Self {
        ObjectPattern::Any
    }

    pub fn pattern(pattern: Pattern) -> Self {
        ObjectPattern::Pattern(pattern)
    }
}

impl Matcher for ObjectPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        if let Some(object) = envelope.as_object() {
            match self {
                ObjectPattern::Any => {
                    vec![vec![envelope.clone(), object.clone()]]
                }
                ObjectPattern::Pattern(pattern) => {
                    if pattern.matches(&object) {
                        vec![vec![envelope.clone(), object.clone()]]
                    } else {
                        vec![]
                    }
                }
            }
        } else {
            return vec![];
        }
    }
}
