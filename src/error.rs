#[derive(Debug)]
pub enum Error {
    /// Compressed envelopes cannot be compressed again.
    AlreadyCompressed,

    /// Elided envelopes compressed or encrypted.
    AlreadyElided,

    /// Envelopes cannot be encrypted twice or compressed.
    AlreadyEncrypted,

    /// Asked for the single assertion with a given predicate when there is more than one.
    AmbiguousPredicate,

    /// Digest did not match.
    InvalidDigest,

    /// General decoding error.
    InvalidFormat,

    /// A recipient that matches the given key was not found.
    InvalidRecipient,

    /// The given SSKR shares were not correct.
    InvalidShares,

    /// The expected digest was not found.
    MissingDigest,

    /// Could not find an assertion with the given predicate.
    NonexistentPredicate,

    /// Cannot uncompress an envelope that is not compressed.
    NotCompressed,

    /// Cannot decrpt an envelope whose subject is not encrypted.
    NotEncrypted,

    /// Cannot unwrap an envelope that is not wrapped.
    NotWrapped,

    /// Could not verify a signature.
    UnverifiedSignature,

    /// Error from the CBOR library.
    CBORError(dcbor::Error),

    /// Error from the crypto library.
    CryptoError(bc_crypto::Error),

    /// Error from the SSKR library.
    SSKRError(bc_components::SSKRError),

    /// Error from the compression component.
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
