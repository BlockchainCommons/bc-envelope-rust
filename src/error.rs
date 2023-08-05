/// Error returned when handling envelopes.
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

    /// Cannot decrypt an envelope whose subject is not encrypted.
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
        let s = match self {
            Error::AlreadyCompressed => "envelopes was already compressed".to_string(),
            Error::AlreadyElided => "envelope was elided, so it cannot be compressed or encrypted".to_string(),
            Error::AlreadyEncrypted => "envelope was already encrypted or compressed, so it cannot be encrypted".to_string(),
            Error::AmbiguousPredicate => "more than one assertion matches the predicate".to_string(),
            Error::InvalidDigest => "digest did not match".to_string(),
            Error::InvalidFormat => "invalid format".to_string(),
            Error::InvalidRecipient => "no recipient matches the given key".to_string(),
            Error::InvalidShares => "the given SSKR shares were not correct".to_string(),
            Error::MissingDigest => "a digest was expected but not found".to_string(),
            Error::NonexistentPredicate => "no assertion matches the predicate".to_string(),
            Error::NotCompressed => "cannot uncompress an envelope that was not compressed".to_string(),
            Error::NotEncrypted => "cannot decrypt an envelope that was not encrypted".to_string(),
            Error::NotWrapped => "cannot unwrap an envelope that was not wrapped".to_string(),
            Error::UnverifiedSignature => "could not verify a signature".to_string(),
            Error::CBORError(err) => format!("{}", err),
            Error::CryptoError(err) => format!("{}", err),
            Error::SSKRError(err) => format!("{}", err),
            Error::CompressedError(err) => format!("{}", err),
        };
        f.write_str(&s)
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
    fn from(_e: Error) -> Self {
        dcbor::Error::InvalidFormat
    }
}
