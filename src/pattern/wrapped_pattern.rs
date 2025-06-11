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
    pub fn any() -> Self { WrappedPattern::Any }

    /// Creates a new `WrappedPattern` that matches a wrapped envelope with a
    /// specific pattern as its subject.
    pub fn subject(pattern: Pattern) -> Self {
        WrappedPattern::Subject(Box::new(pattern))
    }
}

impl Matcher for WrappedPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        if let Some(content) = envelope.subject().unwrap_envelope().ok() {
            match self {
                WrappedPattern::Any => {
                    vec![vec![envelope.clone(), content]]
                }
                WrappedPattern::Subject(pattern) => {
                    if pattern.matches(&content) {
                        vec![vec![envelope.clone(), content]]
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
