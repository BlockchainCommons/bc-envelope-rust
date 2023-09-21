use std::rc::Rc;
use anyhow::bail;
use bc_components::{Compressed, DigestProvider};
use dcbor::preamble::*;

use crate::{Envelope, EnvelopeError};

/// Support for compressing and uncompressing envelopes.
impl Envelope {
    /// Returns the compressed variant of this envelope.
    ///
    /// Returns the same envelope if it is already compressed.
    pub fn compress(self: Rc<Self>) -> Result<Rc<Self>, EnvelopeError> {
        match &*self {
            Envelope::Compressed(_) => Ok(self),
            Envelope::Encrypted(_) => Err(EnvelopeError::AlreadyEncrypted),
            Envelope::Elided(_) => Err(EnvelopeError::AlreadyElided),
            _ => {
                let compressed = Compressed::from_uncompressed_data(self.tagged_cbor().cbor_data(), Some(self.digest().into_owned()));
                Ok(Envelope::new(compressed))
            },
        }
    }

    /// Returns the uncompressed variant of this envelope.
    ///
    /// Returns the same envelope if it is already uncompressed.
    pub fn uncompress(self: Rc<Self>) -> anyhow::Result<Rc<Self>> {
        if let Envelope::Compressed(compressed) = &*self {
            if let Some(digest) = compressed.digest_ref_opt() {
                if digest != self.digest().as_ref() {
                    bail!(EnvelopeError::InvalidDigest);
                }
                let a = compressed.uncompress()?;
                let envelope = Rc::new(Envelope::from_tagged_cbor_data(&a)?);
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
    pub fn compress_subject(self: Rc<Self>) -> Result<Rc<Self>, EnvelopeError> {
        if self.clone().subject().is_compressed() {
            Ok(self)
        } else {
            let subject = self.clone().subject().compress()?;
            Ok(self.replace_subject(subject))
        }
    }

    /// Returns this envelope with its subject uncompressed.
    ///
    /// Returns the same envelope if its subject is already uncompressed.
    pub fn uncompress_subject(self: Rc<Self>) -> anyhow::Result<Rc<Self>> {
        if self.clone().subject().is_compressed() {
            let subject = self.clone().subject().uncompress()?;
            Ok(self.replace_subject(subject))
        } else {
            Ok(self)
        }
    }
}
