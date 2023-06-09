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
    CompressedError(bc_components::CompressedError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<dcbor::Error> for Error {
    fn from(e: dcbor::Error) -> Self {
        Error::CBORError(e)
    }
}

impl From<bc_crypto::Error> for Error {
    fn from(e: bc_crypto::Error) -> Self {
        Error::CryptoError(e)
    }
}

impl From<bc_components::SSKRError> for Error {
    fn from(e: bc_components::SSKRError) -> Self {
        Error::SSKRError(e)
    }
}

impl From<bc_components::CompressedError> for Error {
    fn from(e: bc_components::CompressedError) -> Self {
        Error::CompressedError(e)
    }
}

impl From<Error> for dcbor::Error {
    fn from(_value: Error) -> Self {
        dcbor::Error::InvalidFormat
    }
}
