use thiserror::Error;

/// Error returned when handling envelopes.
#[derive(Debug, Error)]
pub enum EnvelopeError {
    #[cfg(feature = "compress")]
    #[error("envelope was already compressed")]
    AlreadyCompressed,

    #[error("envelope was elided, so it cannot be compressed or encrypted")]
    AlreadyElided,

    #[cfg(feature = "encrypt")]
    #[error("envelope was already encrypted or compressed, so it cannot be encrypted")]
    AlreadyEncrypted,

    #[error("more than one assertion matches the predicate")]
    AmbiguousPredicate,

    #[error("digest did not match")]
    InvalidDigest,

    #[error("invalid format")]
    InvalidFormat,

    #[cfg(feature = "recipient")]
    #[error("no recipient matches the given key")]
    InvalidRecipient,

    #[cfg(feature = "sskr")]
    #[error("the given SSKR shares were not correct")]
    InvalidShares,

    #[error("a digest was expected but not found")]
    MissingDigest,

    #[error("no assertion matches the predicate")]
    NonexistentPredicate,

    #[cfg(feature = "compress")]
    #[error("cannot uncompress an envelope that was not compressed")]
    NotCompressed,

    #[cfg(feature = "encrypt")]
    #[error("cannot decrypt an envelope that was not encrypted")]
    NotEncrypted,

    #[error("cannot unwrap an envelope that was not wrapped")]
    NotWrapped,

    #[cfg(feature = "signature")]
    #[error("could not verify a signature")]
    UnverifiedSignature,

    #[error("the envelope's subject is not a leaf")]
    NotLeaf,

    #[error("the envelope's subject is not an assertion")]
    NotAssertion,

    #[cfg(feature = "known_value")]
    #[error("the envelope's subject is not a known value")]
    NotKnownValue,

    #[cfg(feature = "attachment")]
    #[error("invalid attachment")]
    InvalidAttachment,

    #[cfg(feature = "attachment")]
    #[error("nonexistent attachment")]
    NonexistentAttachment,

    #[cfg(feature = "attachment")]
    #[error("abiguous attachment")]
    AmbiguousAttachment,
}
