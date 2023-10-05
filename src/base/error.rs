use thiserror::Error;

/// Error returned when handling envelopes.
#[derive(Debug, Error)]
pub enum EnvelopeError {
    //
    // Base Specification
    //

    #[error("envelope was elided, so it cannot be compressed or encrypted")]
    AlreadyElided,

    #[error("more than one assertion matches the predicate")]
    AmbiguousPredicate,

    #[error("digest did not match")]
    InvalidDigest,

    #[error("invalid format")]
    InvalidFormat,

    #[error("a digest was expected but not found")]
    MissingDigest,

    #[error("no assertion matches the predicate")]
    NonexistentPredicate,

    #[error("cannot unwrap an envelope that was not wrapped")]
    NotWrapped,

    #[error("the envelope's subject is not a leaf")]
    NotLeaf,

    #[error("the envelope's subject is not an assertion")]
    NotAssertion,


    //
    // Attachments Extension
    //

    #[cfg(feature = "attachment")]
    #[error("invalid attachment")]
    InvalidAttachment,

    #[cfg(feature = "attachment")]
    #[error("nonexistent attachment")]
    NonexistentAttachment,

    #[cfg(feature = "attachment")]
    #[error("abiguous attachment")]
    AmbiguousAttachment,


    //
    // Compression Extension
    //

    #[cfg(feature = "compress")]
    #[error("envelope was already compressed")]
    AlreadyCompressed,

    #[cfg(feature = "compress")]
    #[error("cannot uncompress an envelope that was not compressed")]
    NotCompressed,


    //
    // Symmetric Encryption Extension
    //

    #[cfg(feature = "encrypt")]
    #[error("envelope was already encrypted or compressed, so it cannot be encrypted")]
    AlreadyEncrypted,

    #[cfg(feature = "encrypt")]
    #[error("cannot decrypt an envelope that was not encrypted")]
    NotEncrypted,


    //
    // Known Values Extension
    //

    #[cfg(feature = "known_value")]
    #[error("the envelope's subject is not a known value")]
    NotKnownValue,


    //
    // Public Key Encryption Extension
    //

    #[cfg(feature = "recipient")]
    #[error("no recipient matches the given key")]
    InvalidRecipient,


    //
    // Public Key Signing Extension
    //

    #[cfg(feature = "signature")]
    #[error("could not verify a signature")]
    UnverifiedSignature,


    //
    // SSKR Extension
    //

    #[cfg(feature = "sskr")]
    #[error("invalid SSKR shares")]
    InvalidShares,


    //
    // Types Extension
    //

    #[cfg(feature = "types")]
    #[error("invalid type")]
    InvalidType,

    #[cfg(feature = "types")]
    #[error("ambiguous type")]
    AmbiguousType,
}
