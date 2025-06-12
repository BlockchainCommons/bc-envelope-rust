use crate::{Envelope, pattern::{Matcher, Path}};

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

impl ObscuredPattern {
    /// Creates a new `ObscuredPattern` that matches any obscured element.
    pub fn any() -> Self {
        ObscuredPattern::Any
    }

    /// Creates a new `ObscuredPattern` that matches any elided element.
    pub fn elided() -> Self {
        ObscuredPattern::Elided
    }

    /// Creates a new `ObscuredPattern` that matches any encrypted element.
    pub fn encrypted() -> Self {
        ObscuredPattern::Encrypted
    }

    /// Creates a new `ObscuredPattern` that matches any compressed element.
    pub fn compressed() -> Self {
        ObscuredPattern::Compressed
    }
}

impl Matcher for ObscuredPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        let is_hit = match self {
            ObscuredPattern::Any => envelope.is_obscured(),
            ObscuredPattern::Elided => envelope.is_elided(),
            ObscuredPattern::Encrypted => {
                #[cfg(feature = "encrypt")]
                {
                    envelope.is_encrypted()
                }
                #[cfg(not(feature = "encrypt"))]
                {
                    false
                }
            },
            ObscuredPattern::Compressed => {
                #[cfg(feature = "compress")]
                {
                    envelope.is_compressed()
                }
                #[cfg(not(feature = "compress"))]
                {
                    false
                }
            },
        };

        if is_hit {
            vec![vec![envelope.clone()]]
        } else {
            vec![]
        }
    }
}
