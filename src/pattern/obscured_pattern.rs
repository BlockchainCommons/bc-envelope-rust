use crate::Envelope;
use super::match_pattern::MatchPattern;

/// Pattern for matching obscured elements.
#[derive(Debug, Clone)]
pub enum ObscuredPattern {
    /// Matches any obscured element.
    Any,
    /// Matches any elided element.
    Elided,
    /// Matches any encrypted element.
    Encrypted,
    /// Matches any compressed element.
    Compressed,
}

impl MatchPattern for ObscuredPattern {
    fn matches(&self, envelope: &Envelope) -> bool {
        match self {
            ObscuredPattern::Any => envelope.is_obscured(),
            ObscuredPattern::Elided => envelope.is_elided(),
            ObscuredPattern::Encrypted => envelope.is_encrypted(),
            ObscuredPattern::Compressed => envelope.is_compressed(),
        }
    }
}
