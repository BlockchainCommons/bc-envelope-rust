#![doc(html_root_url = "https://docs.rs/bc-envelope/0.18.2")]
#![warn(rust_2018_idioms)]

//! # Gordian Envelope: A Flexible Container for Structured Data
//!
//! ## Introduction
//!
//! The [Gordian
//! Envelope](https://www.blockchaincommons.com/introduction/Envelope-Intro/)
//! protocol specifies a structured format for hierarchical binary data focused
//! on the ability to transmit it in a privacy-focused way. Envelopes are
//! designed to facilitate "smart documents" and have a number of unique
//! features including: easy representation of a variety of semantic structures,
//! a built-in Merkle-like digest tree, deterministic representation using CBOR,
//! and the ability for the holder of a document to selectively encrypt or elide
//! specific parts of a document without invalidating the document structure
//! including the digest tree, or any cryptographic signatures that rely on it.
//!
//! ## Getting Started
//!
//! ```toml
//! [dependencies]
//! bc-envelope = "0.18.2"
//! ```
//!
//! ## Specification
//!
//! Gordian Envelope is currently specified in [this IETF Internet
//! Draft](https://datatracker.ietf.org/doc/draft-mcnally-envelope/).
//!
//! Envelopes are immutable. You create "mutations" by creating new envelopes
//! from old envelopes.
//!
//! # Basic Envelope Creation
//!
//! * [`Envelope::new`] Creates an envelope with a `subject`.
//! * [`Envelope::new_assertion`] Creates an assertion envelope with a
//!   `predicate` and `object`.
//!
//! # Adding Assertions
//!
//! ### Adding Assertions with a Predicate and Object
//!
//! * [`Envelope::add_assertion`] Adds an assertion to an envelope.
//! * [`Envelope::add_assertion_salted`] Adds an optionally salted assertion to
//!   an envelope.
//! * [`Envelope::add_optional_assertion`] Optionally adds an assertion to an
//!   envelope.
//!
//! ### Adding Assertions with an Assertion Envelope
//!
//! * [`Envelope::add_assertion_envelope`] Adds an assertion envelope to an
//!   envelope.
//! * [`Envelope::add_assertion_envelope_salted`] Adds an optionally salted
//!   assertion envelope to an envelope.
//! * [`Envelope::add_assertion_envelopes`] Adds a vector of assertion envelopes
//!   to an envelope.
//! * [`Envelope::add_optional_assertion_envelope_salted`] Optionally adds an
//!   assertion envelope to an envelope.
//!
//! # Removing and Replacing Assertions
//!
//! * [`Envelope::remove_assertion`] Removes an assertion from an envelope.
//! * [`Envelope::replace_assertion`] Replaces an assertion in an envelope.
//! * [`Envelope::replace_subject`] Replaces the subject of an envelope.
//!
//! # Queries
//!
//! ### Getting the basic parts of an envelope
//!
//! * [`Envelope::subject`] Returns the subject of an envelope.
//! * [`Envelope::predicate`] If the envelope’s subject is an assertion return
//!   its predicate, else return `None`.
//! * [`Envelope::object`] If the envelope’s subject is an assertion return its
//!   object, else return `None`.
//!
//! ### Getting assertions on an envelope
//!
//! * [`Envelope::assertions`] Returns the assertions of an envelope.
//! * [`Envelope::has_assertions`] Returns whether an envelope has assertions.
//! * [`Envelope::assertion`] If the envelope’s subject is an assertion return
//!   it, else return `None`.
//!
//! ### Getting the specific types of an envelope
//!
//! * [`Envelope::leaf`] The envelope’s leaf CBOR object, or `None` if the
//!   envelope is not a leaf.
//! * [`Envelope::known_value`] The envelope’s known value, or `None` if the
//!   envelope is not a known value.
//!
//! ### Determining the type of an envelope
//!
//! * [`Envelope::is_leaf`] Returns whether an envelope is a leaf.
//! * [`Envelope::is_node`] Returns whether an envelope is a node (whether it
//!   has at least one assertion).
//! * [`Envelope::is_wrapped`] Returns whether an envelope is wrapped.
//! * [`Envelope::is_known_value`] Returns whether an envelope is a known value.
//! * [`Envelope::is_assertion`] Returns whether an envelope is an assertion.
//! * [`Envelope::is_encrypted`] Returns whether an envelope is encrypted.
//! * [`Envelope::is_compressed`] Returns whether an envelope is compressed.
//! * [`Envelope::is_elided`] Returns whether an envelope is elided.
//!
//! ### Determining the type of an envelope’s subject
//!
//! * [`Envelope::is_subject_assertion`] Returns whether an envelope’s subject
//!   is an assertion.
//! * [`Envelope::is_subject_encrypted`] Returns whether an envelope’s subject
//!   is encrypted.
//! * [`Envelope::is_subject_compressed`] Returns whether an envelope’s subject
//!   is compressed.
//! * [`Envelope::is_subject_elided`] Returns whether an envelope’s subject is
//!   elided.
//! * [`Envelope::is_subject_obscured`] Returns whether an envelope’s subject is
//!   obscured.
//!
//! ### Getting assertions and parts of assertions
//!
//! * [`Envelope::assertion_with_predicate`] Returns the assertion with the
//!   given predicate.
//! * [`Envelope::assertions_with_predicate`] Returns all assertions with the
//!   given predicate.
//! * [`Envelope::object_for_predicate`] Returns the object of the assertion
//!   with the given predicate.
//! * [`Envelope::objects_for_predicate`] Returns the objects of all assertions
//!   with the matching predicate.
//! * [`Envelope::elements_count`] Returns the number of elements in the
//!   envelope.
//!
//! ### Extracting parts of envelopes as specific types
//!
//! * [`Envelope::extract_subject`] Returns the envelope’s subject, decoded as
//!   the given type.
//! * [`Envelope::extract_object_for_predicate`] Returns the object of the
//!   assertion with the given predicate, decoded as the given type.
//! * [`Envelope::extract_objects_for_predicate`] Returns the objects of all
//!   assertions with the matching predicate, decoded as the given type.
//!
//! ### Other queries
//!
//! * [`Envelope::is_internal`] Returns whether an envelope is internal, that
//!   is, if it has child elements.
//! * [`Envelope::is_obscured`] Returns whether an envelope is obscured (elided,
//!   encrypted, or compressed).
//!
//! # Wrapping and Unwrapping Envelopes
//!
//! * [`Envelope::wrap_envelope`] Wraps an envelope in a new envelope.
//! * [`Envelope::unwrap_envelope`] Unwraps an envelope.
//!
//! # Formatting Envelopes
//!
//! ### Envelope notation
//!
//! * [`Envelope::format`] Formats an envelope in envelope notation.
//! * [`Envelope::format_opt`] Formats an envelope in envelope notation, with
//!   optional annotations.
//!
//! ### Tree notation
//!
//! * [`Envelope::tree_format`] Formats an envelope in envelope tree notation.
//! * [`Envelope::tree_format_with_target`] Formats an envelope in envelope tree
//!   notation, highlighting a target set of elements.
//!
//! ### CBOR diagnostic notation
//!
//! * [`Envelope::diagnostic`] Formats an envelope in CBOR diagnostic notation.
//! * [`Envelope::diagnostic_opt`] Formats an envelope in CBOR diagnostic
//!   notation, with optional annotations.
//!
//! ### CBOR hexadecimal notation
//!
//! * [`Envelope::hex`] Formats an envelope in CBOR hexadecimal notation.
//! * [`Envelope::hex_opt`] Formats an envelope in CBOR hexadecimal notation,
//!   with optional annotations.
//!
//! # Working with the Digest Tree
//!
//! ### Semantic equivalence
//!
//! * [`bc_components::DigestProvider::digest`] Returns the digest of an
//!   envelope.
//! * [`Envelope::digests`] Returns the set of digests contained in the
//!   envelope’s elements, down to the specified level.
//! * [`Envelope::deep_digests`] Returns the set of all digests in the envelope.
//! * [`Envelope::shallow_digests`] Returns the set of all digests in the
//!   envelope, down to its second level.
//! * [`Envelope::is_equivalent_to`] Tests two envelopes for semantic
//!   equivalence.
//!
//! ### Structural identicality
//!
//! * [`Envelope::structural_digest`] Produce a value that will necessarily be
//!   different if two envelopes differ structurally, even if they are
//!   semantically equivalent.
//! * [`Envelope::is_identical_to`] Tests two envelopes for structural equality.
//!
//! # Signing and Verifying Signatures
//!
//! ### Signing
//!
//! * [`Envelope::sign_with`] Creates a signature for the envelope's subject and
//!   returns a new envelope with a `'signed': Signature` assertion.
//! * [`Envelope::sign_with_opt`] Creates a signature for the envelope's subject
//!   and returns a new envelope with a `'signed': Signature` assertion.
//! * [`Envelope::sign_with_keys`] Creates several signatures for the envelope's
//!   subject and returns a new envelope with additional `'signed': Signature`
//!   assertions.
//! * [`Envelope::sign_with_keys_opt`] Creates several signatures for the
//!   envelope's subject and returns a new envelope with additional `'signed':
//!   Signature` assertions.
//! * [`Envelope::sign_with_uncovered_assertions`] Creates a signature for the
//!   envelope's subject and returns a new envelope with a `'signed':
//!   Signature` assertion.
//!
//! ### Verifying by returning a boolean
//!
//! * [`Envelope::is_verified_signature`] Returns whether the given signature is
//!   valid.
//! * [`Envelope::has_signature_from`] Returns whether the envelope's subject
//!   has a valid signature from the given public key.
//! * [`Envelope::has_signatures_from`] Returns whether the envelope's subject
//!   has a set of signatures.
//! * [`Envelope::has_signatures_from_threshold`] Returns whether the envelope's
//!   subject has some threshold of signatures.
//!
//! ### Verifying by returning a result
//!
//! * [`Envelope::verify_signature`] Checks whether the given signature is valid
//!   for the given public key.
//! * [`Envelope::verify_signature_from`] Checks whether the envelope's subject
//!   has a valid signature from the given public key.
//! * [`Envelope::verify_signatures_from`] Checks whether the envelope's subject
//!   has a set of signatures.
//! * [`Envelope::verify_signatures_from_threshold`] Checks whether the
//!   envelope's subject has some threshold of signatures.
//!
//! ### Helpers
//!
//! * [`Envelope::signatures`] Returns an array of `Signature`s from all of the
//!   envelope's `signed` predicates.
//! * [`Envelope::make_signed_assertion`] Convenience constructor for a
//!   `signed: Signature` assertion envelope.
//!
//! # Splitting Envelopes with SSKR
//!
//! * [`Envelope::sskr_split`] Splits the envelope into a set of SSKR shares.
//! * [`Envelope::sskr_join`] Creates a new envelope resulting from the joining
//!   a set of envelopes split by SSKR.
//!
//! # Encryption
//!
//! * [`Envelope::encrypt_subject`] Returns a new envelope with its subject
//!   encrypted.
//! * [`Envelope::decrypt_subject`] Returns a new envelope with its subject
//!   decrypted.
//!
//! # Public Key Encryption
//!
//! * [`Envelope::add_recipient`] Returns a new envelope with an added
//!   `hasRecipient: SealedMessage` assertion.
//! * [`Envelope::recipients`] Returns an array of `SealedMessage`s from all of
//!   the envelope's `hasRecipient` assertions.
//! * [`Envelope::encrypt_subject_to_recipients`] Returns an new envelope with
//!   its subject encrypted and a `hasRecipient`
//! * [`Envelope::encrypt_subject_to_recipient`] Returns a new envelope with its
//!   subject encrypted and a `hasRecipient` assertion added for the
//!   `recipient`.
//! * [`Envelope::decrypt_to_recipient`] Returns a new envelope with its subject
//!   decrypted using the recipient's `PrivateKeyBase`.
//!
//! # Compression
//!
//! * [`Envelope::compress`] Returns the compressed variant of this envelope.
//! * [`Envelope::uncompress`] Returns the uncompressed variant of this
//!   envelope.
//! * [`Envelope::compress_subject`] Returns this envelope with its subject
//!   compressed.
//! * [`Envelope::uncompress_subject`] Returns this envelope with its subject
//!   uncompressed.
//!
//! # Eliding, Encrypting, or Compressing Parts of an Envelope
//!
//! * [`Envelope::elide`] Returns the elided variant of this envelope.
//!
//! * Returns a version of this envelope with the given element(s) elided:
//!     * [`Envelope::elide_removing_set`]
//!     * [`Envelope::elide_removing_array`]
//!     * [`Envelope::elide_removing_target`]
//!
//! * Returns a version with all elements except the given element(s) elided:
//!     * [`Envelope::elide_revealing_set`]
//!     * [`Envelope::elide_revealing_array`]
//!     * [`Envelope::elide_revealing_target`]
//!
//! * As above, but takes a boolean to determine whether to remove or reveal:
//!     * [`Envelope::elide_set`]
//!     * [`Envelope::elide_array`]
//!     * [`Envelope::elide_target`]
//!
//! * Returns a version with the given element(s) elided, encrypted, or
//! compressed:
//!     * [`Envelope::elide_removing_set_with_action`]
//!     * [`Envelope::elide_removing_array_with_action`]
//!     * [`Envelope::elide_removing_target_with_action`]
//!
//! * Returns a version with all elements except the given element(s) elided,
//! encrypted, or compressed:
//!     * [`Envelope::elide_revealing_set_with_action`]
//!     * [`Envelope::elide_revealing_array_with_action`]
//!     * [`Envelope::elide_revealing_target_with_action`]
//!
//! * As above, but takes a boolean to determine whether to remove or reveal:
//!     * [`Envelope::elide_set_with_action`]
//!     * [`Envelope::elide_array_with_action`]
//!     * [`Envelope::elide_target_with_action`]
//!
//! * [`Envelope::unelide`] Returns the unelided variant of this envelope, given
//!   the envelope that was elided.
//!
//! # Decorrelating Envelopes using Salt
//!
//! * [`Envelope::add_salt`] Add a number of bytes of salt generally
//!   proportionate to the size of the object being salted.
//! * [`Envelope::add_salt_with_len`] Add a specified number of bytes of salt.
//! * [`Envelope::add_salt_in_range`] Add a number of bytes of salt chosen
//!   randomly from the given range.
//!
//! # Walking an Envelope's Hierarchy
//!
//! * [`Envelope::walk`] Walk the envelope, calling the visitor function for
//!   each element.
//!
//! # Envelope Expressions
//!
//! ### Constructing Expressions, Requests, and Responses
//!
//! * [`Envelope::new_function`] Creates an envelope with a `«function»`
//!   subject.
//! * [`Envelope::new_parameter`] Creates a new envelope containing a
//!   `❰parameter❱: value` assertion.
//! * [`Envelope::new_optional_parameter`] Optionally adds a `❰parameter❱:
//!   value` assertion to the envelope.
//! * [`Envelope::add_parameter`] Adds a `❰parameter❱: value` assertion to the
//!   envelope.
//! * [`Envelope::add_optional_parameter`] Optionally adds a `❰parameter❱:
//!   value` assertion to the envelope.
//! * [`Envelope::new_request`] Creates an envelope with an `ARID` subject and a
//!   `body: «function»` assertion.
//! * [`Envelope::new_response`] Creates an envelope with an `ARID` subject and a
//!   `result: value` assertion.
//! * [`Envelope::new_response_with_result`] Creates an envelope with an `ARID`
//!   subject and a `result: value` assertion for each provided result.
//! * [`Envelope::new_error_response_with_id`] Creates an envelope with an `ARID`
//!   subject and a `error: value` assertion.
//! * [`Envelope::new_error_response`] Creates an envelope with an `unknown`
//!   subject and a `error: value` assertion.
//!
//! ### Decoding Parameters and Results
//!
//! * [`Envelope::extract_object_for_parameter`] Returns the argument for the
//!   given parameter, decoded as the given type.
//! * [`Envelope::extract_objects_for_parameter`] Returns an array of arguments
//!   for the given parameter, decoded as the given type.
//! * [`Envelope::result`] Returns the object of the `result` predicate.
//! * [`Envelope::results`] Returns the objects of every `result` predicate.
//! * [`Envelope::extract_result`] Returns the object of the `result` predicate,
//!   decoded as the given type.
//! * [`Envelope::extract_results`] Returns the objects of every `result`
//!   predicate, decoded as the given type.
//! * [`Envelope::is_result_ok`] Returns whether the `result` predicate has the
//!   `KnownValue` `.ok`.
//! * [`Envelope::error`] Returns the error value, decoded as the given type.

pub use anyhow::Result;

pub mod base;
pub use base::{Assertion, Envelope, EnvelopeEncodable, EnvelopeError};
pub use base::{register_tags, register_tags_in, FormatContext, GLOBAL_FORMAT_CONTEXT};
pub use base::elide::{self, ObscureAction};

pub mod extension;
pub mod prelude;

mod string_utils;

#[cfg(feature = "signature")]
use bc_components::{Signer, Verifier};
#[cfg(feature = "encrypt")]
use bc_components::SymmetricKey;
#[cfg(feature = "recipient")]
use bc_components::{PrivateKeyBase, PublicKeyBase};

#[cfg(feature = "known_value")]
pub use extension::known_values::{
    self,
    known_value,
    KnownValue,
    KNOWN_VALUES,
    KnownValuesStore,
};

#[cfg(feature = "expression")]
pub use extension::expressions::{
    functions,
    parameters,
    Function,
    Parameter,
    Expression,
    ExpressionBehavior,
    IntoExpression,
    Request,
    RequestBehavior,
    Response,
    ResponseBehavior,
    Event,
    EventBehavior,
};

#[cfg(feature = "encrypt")]
impl Envelope {
    pub fn encrypt(&self, key: &SymmetricKey) -> Envelope {
        self
            .wrap_envelope()
            .encrypt_subject(key)
            .unwrap()
    }

    pub fn decrypt(&self, key: &SymmetricKey) -> Result<Envelope> {
        self
            .decrypt_subject(key)?
            .unwrap_envelope()
    }
}

#[cfg(feature = "signature")]
impl Envelope {
    pub fn sign(&self, signer: &impl Signer) -> Envelope {
        self
            .wrap_envelope()
            .add_signature(signer)
    }

    pub fn verify(&self, verifier: &impl Verifier) -> Result<Envelope> {
        self
            .verify_signature_from(verifier)?
            .unwrap_envelope()
    }
}

#[cfg(feature = "recipient")]
impl Envelope {
    pub fn encrypt_to_recipient(&self, recipient: &PublicKeyBase) -> Envelope {
        self
            .wrap_envelope()
            .encrypt_subject_to_recipient(recipient)
            .unwrap()
    }

    pub fn decrypt_to_recipient(&self, recipient: &PrivateKeyBase) -> Result<Envelope> {
        self
            .decrypt_subject_to_recipient(recipient)?
            .unwrap_envelope()
    }
}

#[cfg(all(feature = "signature", feature = "recipient"))]
impl Envelope {
    pub fn seal(&self, sender: &impl Signer, recipient: &PublicKeyBase) -> Envelope {
        self
            .sign(sender)
            .encrypt_to_recipient(recipient)
    }

    pub fn unseal(&self, sender: &impl Verifier, recipient: &PrivateKeyBase) -> Result<Envelope> {
        self
            .decrypt_to_recipient(recipient)?
            .verify(sender)
    }
}
