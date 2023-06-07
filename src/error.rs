#[derive(Debug)]
pub enum Error {
    AlreadyCompressed,
    AlreadyElided,
    AlreadyEncrypted,
    AmbiguousPredicate,
    InvalidDigest,
    InvalidFormat,
    InvalidKey,
    InvalidRecipient,
    InvalidShares,
    MissingDigest,
    NonexistentPredicate,
    NotCompressed,
    NotEncrypted,
    NotWrapped,
    UnverifiedSignature,
    CBORError(dcbor::Error),
    CryptoError(bc_crypto::Error),
    SSKRError(bc_components::SSKRError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
