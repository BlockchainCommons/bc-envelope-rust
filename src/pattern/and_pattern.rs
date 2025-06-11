use super::{Matcher, Path, Pattern};
use crate::Envelope;

/// A pattern that matches if all contained patterns match.
#[derive(Debug, Clone)]
pub struct AndPattern {
    pub(crate) patterns: Vec<Pattern>,
}

impl AndPattern {
    /// Creates a new `AndPattern` with the given patterns.
    pub fn new(patterns: Vec<Pattern>) -> Self {
        AndPattern { patterns }
    }
}

impl Matcher for AndPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        if self.patterns.iter().all(|pattern| pattern.matches(envelope)) {
            vec![vec![envelope.clone()]]
        } else {
            vec![]
        }
    }
}
