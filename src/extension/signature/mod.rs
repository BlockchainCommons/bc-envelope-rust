//! # Signature Extension
//! 
//! This module provides functionality for digitally signing Envelopes and verifying signatures,
//! with optional metadata support.
//!
//! The signature extension allows:
//! - Signing envelope subjects to validate their authenticity
//! - Adding metadata to signatures (e.g., signer identity, date, purpose)
//! - Verification of signatures, both with and without metadata
//! - Support for multiple signatures on a single envelope
//!
//! ## Core Concepts
//!
//! 1. **Basic Signatures**: Sign an envelope's subject to verify its authenticity, without modifying its content
//! 2. **Signatures with Metadata**: Add metadata to a signature that is itself signed with the same key
//! 3. **Multi-Signatures**: Support for multiple signatures on a single envelope, each potentially with its own metadata
//!
//! ## Signature Approaches
//!
//! The signature module supports two main approaches:
//!
//! 1. **Subject-Only Signing**: When `add_signature()` is called on an envelope, only the subject is signed
//! 2. **Wrapped Envelope Signing**: When `sign()` is called, the entire envelope (subject + assertions) is wrapped and then signed
//!
//! ## Usage Examples
//!
//! ### Basic Signature Example
//!
//! ```ignore
//! use bc_envelope::prelude::*;
//! 
//! # fn main() -> anyhow::Result<()> {
//! # // In a real application, you would use proper key generation
//! # // The following would be your actual key variables from your application
//! # let alice_private_key = /* your private key */;
//! # let alice_public_key = /* your public key */;
//! # let bob_public_key = /* another public key */;
//! 
//! // Create and sign an envelope
//! let envelope = Envelope::new("Hello.")
//!     .add_signature(&alice_private_key);
//!
//! // The structure is "Hello." ['signed': Signature]
//! 
//! // Verify the signature
//! assert!(envelope.has_signature_from(&alice_public_key)?);
//! assert!(!envelope.has_signature_from(&bob_public_key)?);
//!
//! // Extract the verified subject
//! let verified = envelope.verify_signature_from(&alice_public_key)?;
//! let message = verified.extract_subject::<String>()?;
//! assert_eq!(message, "Hello.");
//! # Ok(())
//! # }
//! ```
//!
//! ### Signature with Metadata Example
//!
//! ```ignore
//! use bc_envelope::prelude::*;
//! use known_values::NOTE;
//!
//! # fn main() -> anyhow::Result<()> {
//! # // In a real application, you would use proper key generation
//! # // The following would be your actual key variables from your application
//! # let alice_private_key = /* your private key */;
//! # let alice_public_key = /* your public key */;
//! 
//! // Create and sign an envelope with metadata
//! let metadata = SignatureMetadata::new()
//!     .with_assertion(NOTE, "Alice signed this.");
//!
//! let envelope = Envelope::new("Hello.")
//!     .wrap_envelope()
//!     .add_signature_opt(&alice_private_key, None, Some(metadata));
//!
//! // The structure is:
//! // {
//! //     "Hello."
//! // } [
//! //     'signed': {
//! //         Signature [
//! //             'note': "Alice signed this."
//! //         ]
//! //     } [
//! //         'signed': Signature
//! //     ]
//! // ]
//!
//! // Verify signature and extract metadata
//! let (verified_envelope, metadata_envelope) = envelope.verify_returning_metadata(&alice_public_key)?;
//!
//! // Extract the note from metadata
//! let note = metadata_envelope.object_for_predicate(NOTE)?.extract_subject::<String>()?;
//! assert_eq!(note, "Alice signed this.");
//!
//! // Read the original message
//! let message = verified_envelope.extract_subject::<String>()?;
//! assert_eq!(message, "Hello.");
//! # Ok(())
//! # }
//! ```
//!
//! ## Signature Verification Workflow
//!
//! 1. For a simple signature, verification checks if the signature matches the subject's digest
//! 2. For a signature with metadata:
//!    - The outer signature is verified against the wrapped metadata envelope 
//!    - The inner signature (subject of the metadata envelope) is verified against the original envelope's subject
//!    - Both signatures must be made with the same key
//!
//! ## Implementation Details
//!
//! The signature system is implemented using cryptographic primitives from the `bc-components` crate. The signature
//! extensions make use of the `Signer` and `Verifier` traits to provide a flexible interface for different
//! signature algorithms.

pub mod signature_impl;
pub mod signature_metadata;
pub use signature_metadata::SignatureMetadata;
