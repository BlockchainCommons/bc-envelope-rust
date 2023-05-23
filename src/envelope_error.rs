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
