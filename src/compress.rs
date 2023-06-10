use std::rc::Rc;
use bc_components::{Compressed, DigestProvider};
use dcbor::{CBOREncodable, CBORDecodable};

use crate::{Envelope, Error};

impl Envelope {
    /// Returns the compressed variant of this envelope.
    ///
    /// Returns the same envelope if it is already compressed.
    pub fn compress(self: Rc<Self>) -> Result<Rc<Self>, Error> {
        match &*self {
            Envelope::Compressed(_) => Ok(self),
            Envelope::Encrypted(_) => Err(Error::AlreadyEncrypted),
            Envelope::Elided(_) => Err(Error::AlreadyElided),
            _ => {
                let compressed = Compressed::from_uncompressed_data(self.cbor_data(), Some(self.digest().into_owned()));
                Ok(Envelope::new(compressed))
            },
        }
    }

    /// Returns the uncompressed variant of this envelope.
    ///
    /// Returns the same envelope if it is already uncompressed.
    pub fn uncompress(self: Rc<Self>) -> Result<Rc<Self>, Error> {
        if let Envelope::Compressed(compressed) = &*self {
            if let Some(digest) = compressed.digest_ref_opt() {
                if digest != self.digest().as_ref() {
                    return Err(Error::InvalidDigest);
                }
                let a = compressed.uncompress()?;
                let envelope = Rc::new(Envelope::from_cbor_data(&a)?);
                if envelope.digest().as_ref() != digest {
                    return Err(Error::InvalidDigest);
                }
                Ok(envelope)
            } else {
                Err(Error::MissingDigest)
            }
        } else {
            Err(Error::NotCompressed)
        }
    }
}

impl Envelope {
    /// Returns this envelope with its subject compressed.
    ///
    /// Returns the same envelope if its subject is already compressed.
    pub fn compress_subject(self: Rc<Self>) -> Result<Rc<Self>, Error> {
        if self.clone().subject().is_compressed() {
            Ok(self)
        } else {
            let subject = self.clone().subject().compress()?;
            Ok(self.replace_subject(subject))
        }
    }

    /// Returs this envelope with its subject uncompressed.
    ///
    /// Returns the same envelope if its subject is already uncompressed.
    pub fn uncompress_subject(self: Rc<Self>) -> Result<Rc<Self>, Error> {
        if self.clone().subject().is_compressed() {
            let subject = self.clone().subject().uncompress()?;
            Ok(self.replace_subject(subject))
        } else {
            Ok(self)
        }
    }
}
