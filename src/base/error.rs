use thiserror::Error;
use bc_components::Error as ComponentsError;

/// Error types returned when operating on Gordian Envelopes.
///
/// These errors capture various conditions that can occur when working with
/// envelopes, including structure validation, operation constraints, and
/// extension-specific errors.
///
/// The errors are organized by category, reflecting the base envelope
/// specification and various extensions defined in the Gordian Envelope
/// Internet Draft and Blockchain Commons Research (BCR) documents.
#[derive(Debug, Error)]
pub enum Error {
    //
    // Base Specification
    /// Returned when attempting to compress or encrypt an envelope that has
    /// already been elided.
    ///
    /// This error occurs because an elided envelope only contains a digest
    /// reference and no longer has a subject that can be compressed or
    /// encrypted.
    #[error("envelope was elided, so it cannot be compressed or encrypted")]
    AlreadyElided,

    /// Returned when attempting to retrieve an assertion by predicate, but
    /// multiple matching assertions exist.
    ///
    /// For queries that expect a single result (like `object_for_predicate`),
    /// having multiple matching assertions is ambiguous and requires more
    /// specific targeting.
    #[error("more than one assertion matches the predicate")]
    AmbiguousPredicate,

    /// Returned when a digest validation fails.
    ///
    /// This can occur when unwrapping an envelope, verifying signatures, or
    /// other operations that rely on the integrity of envelope digests.
    #[error("digest did not match")]
    InvalidDigest,

    /// Returned when an envelope's format is invalid.
    ///
    /// This typically occurs during parsing or decoding of an envelope from
    /// CBOR.
    #[error("invalid format")]
    InvalidFormat,

    /// Returned when a digest is expected but not found.
    ///
    /// This can occur when working with envelope structures that require digest
    /// information, such as when working with elided envelopes.
    #[error("a digest was expected but not found")]
    MissingDigest,

    /// Returned when attempting to retrieve an assertion by predicate, but no
    /// matching assertion exists.
    ///
    /// This error occurs with functions like `object_for_predicate` when the
    /// specified predicate doesn't match any assertion in the envelope.
    #[error("no assertion matches the predicate")]
    NonexistentPredicate,

    /// Returned when attempting to unwrap an envelope that wasn't wrapped.
    ///
    /// This error occurs when calling `Envelope::try_unwrap` on an
    /// envelope that doesn't have the wrapped format.
    #[error("cannot unwrap an envelope that was not wrapped")]
    NotWrapped,

    /// Returned when expecting an envelope's subject to be a leaf, but it
    /// isn't.
    ///
    /// This error occurs when calling methods that require access to a leaf
    /// value but the envelope's subject is an assertion, node, or elided.
    #[error("the envelope's subject is not a leaf")]
    NotLeaf,

    /// Returned when expecting an envelope's subject to be an assertion, but it
    /// isn't.
    ///
    /// This error occurs when calling methods that require an assertion
    /// structure but the envelope's subject has a different format.
    #[error("the envelope's subject is not an assertion")]
    NotAssertion,

    /// Returned
    #[error("assertion must be a map with exactly one element")]
    InvalidAssertion,

    //
    // Attachments Extension
    /// Returned when an attachment's format is invalid.
    ///
    /// This error occurs when an envelope contains an attachment with an
    /// invalid structure according to the Envelope Attachment specification
    /// (BCR-2023-006).
    #[cfg(feature = "attachment")]
    #[error("invalid attachment")]
    InvalidAttachment,

    /// Returned when an attachment is requested but does not exist.
    ///
    /// This error occurs when attempting to retrieve an attachment by ID that
    /// doesn't exist in the envelope.
    #[cfg(feature = "attachment")]
    #[error("nonexistent attachment")]
    NonexistentAttachment,

    /// Returned when multiple attachments match a single query.
    ///
    /// This error occurs when multiple attachments have the same ID, making
    /// it ambiguous which attachment should be returned.
    #[cfg(feature = "attachment")]
    #[error("abiguous attachment")]
    AmbiguousAttachment,

    //
    // Compression Extension
    /// Returned when attempting to compress an envelope that is already
    /// compressed.
    ///
    /// This error occurs when calling compression functions on an envelope that
    /// already has compressed content, as defined in BCR-2023-005.
    #[cfg(feature = "compress")]
    #[error("envelope was already compressed")]
    AlreadyCompressed,

    /// Returned when attempting to uncompress an envelope that is not
    /// compressed.
    ///
    /// This error occurs when calling uncompression functions on an envelope
    /// that doesn't contain compressed content.
    #[cfg(feature = "compress")]
    #[error("cannot uncompress an envelope that was not compressed")]
    NotCompressed,

    //
    // Symmetric Encryption Extension
    /// Returned when attempting to encrypt an envelope that is already
    /// encrypted or compressed.
    ///
    /// This error occurs to prevent multiple layers of encryption or encryption
    /// of compressed data, which could reduce security, as defined in
    /// BCR-2023-004.
    #[cfg(feature = "encrypt")]
    #[error(
        "envelope was already encrypted or compressed, so it cannot be encrypted"
    )]
    AlreadyEncrypted,

    /// Returned when attempting to decrypt an envelope that is not encrypted.
    ///
    /// This error occurs when calling decryption functions on an envelope that
    /// doesn't contain encrypted content.
    #[cfg(feature = "encrypt")]
    #[error("cannot decrypt an envelope that was not encrypted")]
    NotEncrypted,

    //
    // Known Values Extension
    /// Returned when expecting an envelope's subject to be a known value, but
    /// it isn't.
    ///
    /// This error occurs when calling methods that require a known value (as
    /// defined in BCR-2023-003) but the envelope's subject is a different
    /// type.
    #[cfg(feature = "known_value")]
    #[error("the envelope's subject is not a known value")]
    NotKnownValue,

    //
    // Public Key Encryption Extension
    /// Returned when attempting to decrypt an envelope with a recipient that
    /// doesn't match.
    ///
    /// This error occurs when trying to use a private key to decrypt an
    /// envelope that wasn't encrypted for the corresponding public key.
    #[cfg(feature = "recipient")]
    #[error("unknown recipient")]
    UnknownRecipient,

    //
    // Encrypted Key Extension
    /// Returned when attempting to decrypt an envelope with a secret that
    /// doesn't match.
    ///
    /// This error occurs when trying to use a secret that does not correspond
    /// to the expected recipient, preventing successful decryption.
    #[cfg(feature = "secret")]
    #[error("secret not found")]
    UnknownSecret,

    //
    // Public Key Signing Extension
    /// Returned when a signature verification fails.
    ///
    /// This error occurs when a signature does not validate against its
    /// purported public key.
    #[cfg(feature = "signature")]
    #[error("could not verify a signature")]
    UnverifiedSignature,

    /// Returned when the outer signature object type is not `Signature`.
    #[cfg(feature = "signature")]
    #[error("unexpected outer signature object type")]
    InvalidOuterSignatureType,

    /// Returned when the inner signature object type is not `Signature`.
    #[cfg(feature = "signature")]
    #[error("unexpected inner signature object type")]
    InvalidInnerSignatureType,

    /// Returned when the inner signature is not made with the same key as the
    /// outer signature.
    #[cfg(feature = "signature")]
    #[error("inner signature not made with same key as outer signature")]
    UnverifiedInnerSignature,

    /// Returned when the signature object is not a `Signature`.
    #[cfg(feature = "signature")]
    #[error("unexpected signature object type")]
    InvalidSignatureType,

    //
    // SSKR Extension
    /// Returned when SSKR shares are invalid or insufficient for
    /// reconstruction.
    ///
    /// This error occurs when attempting to join SSKR shares that are
    /// malformed, from different splits, or insufficient to meet the
    /// recovery threshold.
    #[cfg(feature = "sskr")]
    #[error("invalid SSKR shares")]
    InvalidShares,

    //
    // Types Extension
    /// Returned when an envelope contains an invalid type.
    ///
    /// This error occurs when an envelope's type information doesn't match
    /// the expected format or value.
    #[cfg(feature = "types")]
    #[error("invalid type")]
    InvalidType,

    /// Returned when an envelope contains ambiguous type information.
    ///
    /// This error occurs when multiple type assertions exist that conflict
    /// with each other or create ambiguity about the envelope's type.
    #[cfg(feature = "types")]
    #[error("ambiguous type")]
    AmbiguousType,

    //
    // Expressions Extension
    /// Returned when a response envelope has an unexpected ID.
    ///
    /// This error occurs when processing a response envelope and the ID doesn't
    /// match the expected request ID, as defined in BCR-2023-012.
    #[cfg(feature = "expression")]
    #[error("unexpected response ID")]
    UnexpectedResponseID,

    /// Returned when a response envelope is invalid.
    #[cfg(feature = "expression")]
    #[error("invalid response")]
    InvalidResponse,

    #[cfg(feature = "sskr")]
    #[error("sskr error: {0}")]
    SSKR(#[from] bc_components::SSKRError),

    /// dcbor error
    #[error("dcbor error: {0}")]
    DCBOR(#[from] dcbor::Error),

    /// Components error
    #[error("components error: {0}")]
    Components(#[from] ComponentsError),

    /// General error
    #[error("general error: {0}")]
    General(String),
}

impl Error {
    pub fn msg(msg: impl Into<String>) -> Self {
        Error::General(msg.into())
    }
}

impl From<Error> for dcbor::Error {
    fn from(error: Error) -> dcbor::Error {
        match error {
            Error::DCBOR(err) => err,
            _ => dcbor::Error::Custom(error.to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
