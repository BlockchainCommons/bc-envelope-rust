use dcbor::CBORError;

#[derive(Debug)]
pub enum EnvelopeError {
    InvalidDigest,
    InvalidFormat,
    MissingDigest,
    NonexistentPredicate,
    AmbiguousPredicate,
    CBORError(CBORError),
}
