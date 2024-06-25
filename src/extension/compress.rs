use anyhow::{bail, Result};
use bc_components::{Compressed, DigestProvider};
use dcbor::prelude::*;

use crate::{Envelope, EnvelopeError, base::envelope::EnvelopeCase};

/// Support for compressing and uncompressing envelopes.
impl Envelope {
    /// Returns the compressed variant of this envelope.
    ///
    /// Returns the same envelope if it is already compressed.
    pub fn compress(&self) -> Result<Self> {
        match self.case() {
            EnvelopeCase::Compressed(_) => Ok(self.clone()),
            #[cfg(feature = "encrypt")]
            EnvelopeCase::Encrypted(_) => bail!(EnvelopeError::AlreadyEncrypted),
            EnvelopeCase::Elided(_) => bail!(EnvelopeError::AlreadyElided),
            _ => {
                let compressed = Compressed::from_uncompressed_data(self.tagged_cbor().to_cbor_data(), Some(self.digest().into_owned()));
                Ok(compressed.try_into()?)
            },
        }
    }

    /// Returns the uncompressed variant of this envelope.
    ///
    /// Returns the same envelope if it is already uncompressed.
    pub fn uncompress(&self) -> Result<Self> {
        if let EnvelopeCase::Compressed(compressed) = self.case() {
            if let Some(digest) = compressed.digest_ref_opt() {
                if digest != self.digest().as_ref() {
                    bail!(EnvelopeError::InvalidDigest);
                }
                let uncompressed_data = compressed.uncompress()?;
                let envelope = Envelope::from_tagged_cbor_data(uncompressed_data)?;
                if envelope.digest().as_ref() != digest {
                    bail!(EnvelopeError::InvalidDigest);
                }
                Ok(envelope)
            } else {
                bail!(EnvelopeError::MissingDigest)
            }
        } else {
            bail!(EnvelopeError::NotCompressed)
        }
    }

    /// Returns this envelope with its subject compressed.
    ///
    /// Returns the same envelope if its subject is already compressed.
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
    /// Returns the same envelope if its subject is already uncompressed.
    pub fn uncompress_subject(&self) -> Result<Self> {
        if self.subject().is_compressed() {
            let subject = self.subject().uncompress()?;
            Ok(self.replace_subject(subject))
        } else {
            Ok(self.clone())
        }
    }
}
