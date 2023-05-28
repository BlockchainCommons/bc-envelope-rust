#[derive(Debug)]
pub enum Error {
    AlreadyCompressed,
    AlreadyElided,
    AlreadyEncrypted,
    AmbiguousPredicate,
    CBORError(dcbor::Error),
    InvalidDigest,
    InvalidFormat,
    InvalidKey,
    MissingDigest,
    NonexistentPredicate,
    NotCompressed,
    NotEncrypted,
    NotWrapped,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
