use thiserror::Error;

/// Error returned when handling envelopes.
#[derive(Debug, Error)]
pub enum EnvelopeError {
    #[error("envelope was already compressed")]
    AlreadyCompressed,

    #[error("envelope was elided, so it cannot be compressed or encrypted")]
    AlreadyElided,

    #[error("envelope was already encrypted or compressed, so it cannot be encrypted")]
    AlreadyEncrypted,

    #[error("more than one assertion matches the predicate")]
    AmbiguousPredicate,

    #[error("digest did not match")]
    InvalidDigest,

    #[error("invalid format")]
    InvalidFormat,

    #[error("no recipient matches the given key")]
    InvalidRecipient,

    #[error("the given SSKR shares were not correct")]
    InvalidShares,

    #[error("a digest was expected but not found")]
    MissingDigest,

    #[error("no assertion matches the predicate")]
    NonexistentPredicate,

    #[error("cannot uncompress an envelope that was not compressed")]
    NotCompressed,

    #[error("cannot decrypt an envelope that was not encrypted")]
    NotEncrypted,

    #[error("cannot unwrap an envelope that was not wrapped")]
    NotWrapped,

    #[error("could not verify a signature")]
    UnverifiedSignature,
}
