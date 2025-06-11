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
    fn paths(&self, envelope: &Envelope) -> impl Iterator<Item = Path> {
        if self.patterns.iter().all(|pattern| pattern.matches(envelope)) {
            Some(Vec::from_iter([envelope.subject()])).into_iter()
        } else {
            None.into_iter()
        }
    }
}
