use crate::{
    Envelope,
    pattern::{Matcher, Path},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum WrappedPattern {
    /// Matches any wrapped envelope.
    Any,
    /// Matches a wrapped envelope, and adds the unwrapped envelope to the path.
    Unwrap,
}

impl WrappedPattern {
    /// Creates a new `WrappedPattern` that matches any wrapped envelope.
    pub fn any() -> Self { WrappedPattern::Any }

    /// Creates a new `WrappedPattern` that matches a wrapped envelope and
    /// unwraps it.
    pub fn unwrap() -> Self { WrappedPattern::Unwrap }
}

impl Matcher for WrappedPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        if envelope.subject().is_wrapped() {
            match self {
                WrappedPattern::Any => {
                    vec![vec![envelope.clone()]]
                }
                WrappedPattern::Unwrap => {
                    if let Ok(unwrapped) = envelope.subject().unwrap_envelope()
                    {
                        vec![vec![unwrapped]]
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
