//! Extension for compressing and uncompressing envelopes.
//!
//! This module provides functionality for compressing envelopes to reduce their
//! size while maintaining their digests. Unlike elision, which removes content,
//! compression preserves all the information in the envelope but represents it
//! more efficiently.
//!
//! Compression is implemented using the DEFLATE algorithm and preserves the
//! envelope's digest, making it compatible with the envelope's hierarchical
//! digest tree structure. This means parts of an envelope can be compressed
//! without invalidating signatures or breaking the digest tree.
//!
//! # Examples
//!
//! ```
//! use bc_envelope::prelude::*;
//!
//! // Create an envelope with some larger, compressible content
//! let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";
//! let envelope = Envelope::new(lorem);
//!
//! // Compress the envelope
//! let compressed = envelope.compress().unwrap();
//!
//! // The compressed envelope has the same digest as the original
//! assert_eq!(envelope.digest(), compressed.digest());
//!
//! // But it takes up less space when serialized
//! assert!(compressed.to_cbor_data().len() < envelope.to_cbor_data().len());
//!
//! // The envelope can be uncompressed to recover the original content
//! let uncompressed = compressed.uncompress().unwrap();
//! assert_eq!(uncompressed.extract_subject::<String>().unwrap(), lorem);
//! ```

use anyhow::{Result, bail};
use bc_components::{Compressed, DigestProvider};
use dcbor::prelude::*;

use crate::{Envelope, Error, base::envelope::EnvelopeCase};

/// Support for compressing and uncompressing envelopes.
impl Envelope {
    /// Returns a compressed version of this envelope.
    ///
    /// This method compresses the envelope using the DEFLATE algorithm,
    /// creating a more space-efficient representation while preserving the
    /// envelope's digest and semantic content. The compressed envelope
    /// maintains the same digest as the original, ensuring compatibility
    /// with the envelope's digest tree structure.
    ///
    /// When an envelope is compressed, the entire envelope structure (including
    /// its subject and assertions) is compressed as a single unit. The
    /// compression preserves all the information but reduces the size of
    /// the serialized envelope.
    ///
    /// # Returns
    ///
    /// A Result containing the compressed envelope or an error.
    ///
    /// # Errors
    ///
    /// - Returns `EnvelopeError::AlreadyEncrypted` if the envelope is already
    ///   encrypted
    /// - Returns `EnvelopeError::AlreadyElided` if the envelope is already
    ///   elided
    /// - May return various compression-related errors
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Create an envelope with some content
    /// let text = "This is a fairly long text that will benefit from compression.
    ///            The longer the text, the more effective compression becomes.
    ///            DEFLATE works well on repetitive content like this.";
    /// let envelope = Envelope::new(text);
    ///
    /// // Compress the envelope
    /// let compressed = envelope.compress().unwrap();
    ///
    /// // Check that the compressed version has the same digest
    /// assert_eq!(envelope.digest(), compressed.digest());
    ///
    /// // Verify that the compressed version takes less space
    /// assert!(compressed.to_cbor_data().len() < envelope.to_cbor_data().len());
    /// ```
    pub fn compress(&self) -> Result<Self> {
        match self.case() {
            EnvelopeCase::Compressed(_) => Ok(self.clone()),
            #[cfg(feature = "encrypt")]
            EnvelopeCase::Encrypted(_) => bail!(Error::AlreadyEncrypted),
            EnvelopeCase::Elided(_) => bail!(Error::AlreadyElided),
            _ => {
                let compressed = Compressed::from_uncompressed_data(
                    self.tagged_cbor().to_cbor_data(),
                    Some(self.digest().into_owned()),
                );
                Ok(compressed.try_into()?)
            }
        }
    }

    /// Returns the uncompressed variant of this envelope.
    ///
    /// This method reverses the compression process, restoring the envelope to
    /// its original uncompressed form. The uncompressed envelope will have
    /// the same digest as the compressed version.
    ///
    /// # Returns
    ///
    /// A Result containing the uncompressed envelope or an error.
    ///
    /// # Errors
    ///
    /// - Returns `EnvelopeError::NotCompressed` if the envelope is not
    ///   compressed
    /// - Returns `EnvelopeError::MissingDigest` if the compressed envelope is
    ///   missing its digest
    /// - Returns `EnvelopeError::InvalidDigest` if the decompressed data
    ///   doesn't match the expected digest
    /// - May return various decompression-related errors
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Create and compress an envelope
    /// let original = Envelope::new("Hello, world!");
    /// let compressed = original.compress().unwrap();
    ///
    /// // Uncompress it
    /// let uncompressed = compressed.uncompress().unwrap();
    ///
    /// // The uncompressed envelope should match the original
    /// assert_eq!(
    ///     uncompressed.extract_subject::<String>().unwrap(),
    ///     "Hello, world!"
    /// );
    /// assert_eq!(uncompressed.digest(), original.digest());
    ///
    /// // Trying to uncompress a non-compressed envelope fails
    /// assert!(original.uncompress().is_err());
    /// ```
    pub fn uncompress(&self) -> Result<Self> {
        if let EnvelopeCase::Compressed(compressed) = self.case() {
            if let Some(digest) = compressed.digest_ref_opt() {
                if digest != self.digest().as_ref() {
                    bail!(Error::InvalidDigest);
                }
                let uncompressed_data = compressed.uncompress()?;
                let envelope =
                    Envelope::from_tagged_cbor_data(uncompressed_data)?;
                if envelope.digest().as_ref() != digest {
                    bail!(Error::InvalidDigest);
                }
                Ok(envelope)
            } else {
                bail!(Error::MissingDigest)
            }
        } else {
            bail!(Error::NotCompressed)
        }
    }

    /// Returns this envelope with its subject compressed.
    ///
    /// Unlike `compress()` which compresses the entire envelope, this method
    /// only compresses the subject of the envelope, leaving the assertions
    /// uncompressed. This is useful when you want to compress a large
    /// subject while keeping the assertions readable and accessible.
    ///
    /// # Returns
    ///
    /// A Result containing a new envelope with a compressed subject, or an
    /// error.
    ///
    /// # Errors
    ///
    /// May return errors from the compression process.
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Create an envelope with a large subject and some assertions
    /// let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt.";
    /// let envelope = Envelope::new(lorem)
    ///     .add_assertion("note", "This is a metadata note");
    ///
    /// // Compress just the subject
    /// let subject_compressed = envelope.compress_subject().unwrap();
    ///
    /// // The envelope's digest is preserved
    /// assert_eq!(envelope.digest(), subject_compressed.digest());
    ///
    /// // The subject is now compressed
    /// assert!(subject_compressed.subject().is_compressed());
    ///
    /// // But the assertions are still directly accessible
    /// let note = subject_compressed.object_for_predicate("note").unwrap();
    /// assert_eq!(note.extract_subject::<String>().unwrap(), "This is a metadata note");
    /// ```
    pub fn compress_subject(&self) -> Result<Self> {
        if self.subject().is_compressed() {
            Ok(self.clone())
        } else {
            let subject = self.subject().compress()?;
            Ok(self.replace_subject(subject))
        }
    }

    /// Returns this envelope with its subject uncompressed.
    ///
    /// This method reverses the effect of `compress_subject()`, uncompressing
    /// the subject of the envelope while leaving the rest of the envelope
    /// unchanged.
    ///
    /// # Returns
    ///
    /// A Result containing a new envelope with an uncompressed subject, or an
    /// error.
    ///
    /// # Errors
    ///
    /// May return errors from the decompression process.
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Create an envelope and compress its subject
    /// let original =
    ///     Envelope::new("Hello, world!").add_assertion("note", "Test note");
    /// let compressed = original.compress_subject().unwrap();
    ///
    /// // Verify the subject is compressed
    /// assert!(compressed.subject().is_compressed());
    ///
    /// // Uncompress the subject
    /// let uncompressed = compressed.uncompress_subject().unwrap();
    ///
    /// // Verify the subject is now uncompressed
    /// assert!(!uncompressed.subject().is_compressed());
    ///
    /// // The content should match the original
    /// assert_eq!(
    ///     uncompressed.extract_subject::<String>().unwrap(),
    ///     "Hello, world!"
    /// );
    /// ```
    pub fn uncompress_subject(&self) -> Result<Self> {
        if self.subject().is_compressed() {
            let subject = self.subject().uncompress()?;
            Ok(self.replace_subject(subject))
        } else {
            Ok(self.clone())
        }
    }
}
