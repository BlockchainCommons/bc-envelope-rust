use dcbor::CBORError;

#[derive(Debug)]
pub enum EnvelopeError {
    InvalidDigest,
    InvalidFormat,
    MissingDigest,
    NonexistentPredicate,
    AmbiguousPredicate,
    NotWrapped,
    CBORError(CBORError),
}

impl std::fmt::Display for EnvelopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for EnvelopeError {}
