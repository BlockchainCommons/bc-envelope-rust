use super::{Matcher, Path, Pattern};
use crate::Envelope;

#[derive(Debug, Clone)]
pub enum WrappedPattern {
    /// Matches any wrapped envelope.
    Any,
    /// Matches a wrapped envelope with a specific pattern as its subject.
    Subject(Box<Pattern>),
}

impl WrappedPattern {
    /// Creates a new `WrappedPattern` that matches any wrapped envelope.
    pub fn any() -> Self {
        WrappedPattern::Any
    }

    /// Creates a new `WrappedPattern` that matches a wrapped envelope with a specific pattern as its subject.
    pub fn subject(pattern: Pattern) -> Self {
        WrappedPattern::Subject(Box::new(pattern))
    }
}

impl Matcher for WrappedPattern {
    fn paths(&self, envelope: &Envelope) -> impl Iterator<Item = Path> {
        if let Some(content) = envelope.subject().unwrap_envelope().ok() {
            match self {
                WrappedPattern::Any => Some(Vec::from_iter([envelope.clone(), content])).into_iter(),
                WrappedPattern::Subject(pattern) => {
                    if pattern.is_match(&content) {
                        Some(Vec::from_iter([envelope.clone(), content])).into_iter()
                    } else {
                        None.into_iter()
                    }
                }
            }
        } else {
            None.into_iter()
        }
    }
}
