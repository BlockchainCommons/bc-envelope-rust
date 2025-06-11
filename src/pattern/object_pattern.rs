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
    fn paths(&self, envelope: &Envelope) -> impl Iterator<Item = Path> {
        if let Some(object) = envelope.as_object() {
            match self {
                ObjectPattern::Any => {
                    Some(Vec::from_iter([envelope.clone(), object.clone()])).into_iter()
                }
                ObjectPattern::Pattern(pattern) => {
                    if pattern.matches(&object) {
                        Some(Vec::from_iter([envelope.clone(), object.clone()])).into_iter()
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
