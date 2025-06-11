use super::{Matcher, Path, Pattern};
use crate::Envelope;

/// A pattern that matches if any contained pattern matches.
#[derive(Debug, Clone)]
pub struct OrPattern {
    pub(crate) patterns: Vec<Pattern>,
}

impl OrPattern {
    /// Creates a new `OrPattern` with the given patterns.
    pub fn new(patterns: Vec<Pattern>) -> Self { OrPattern { patterns } }
}

impl Matcher for OrPattern {
    fn paths(&self, envelope: &Envelope) -> impl Iterator<Item = Path> {
        if self
            .patterns
            .iter()
            .any(|pattern| pattern.is_match(envelope))
        {
            Some(Vec::from_iter([envelope.subject()])).into_iter()
        } else {
            None.into_iter()
        }
    }
}
