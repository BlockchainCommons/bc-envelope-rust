use crate::pattern::{Matcher, Path, Pattern};
use crate::Envelope;

/// A pattern that matches if any contained pattern matches.
#[derive(Debug, Clone)]
pub struct OrPattern {
    pub patterns: Vec<Pattern>,
}

impl OrPattern {
    /// Creates a new `OrPattern` with the given patterns.
    pub fn new(patterns: Vec<Pattern>) -> Self { OrPattern { patterns } }
}

impl Matcher for OrPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        if self
            .patterns
            .iter()
            .any(|pattern| pattern.matches(envelope))
        {
            vec![vec![envelope.clone()]]
        } else {
            vec![]
        }
    }
}
