use std::rc::Rc;

use bc_components::DigestProvider;
use dcbor::prelude::*;

use crate::Envelope;
use anyhow::{anyhow, bail};

pub trait CheckEncoding {
    fn check_encoding(self) -> Result<Self, anyhow::Error>
    where
        Self: Sized;
}

impl CheckEncoding for Rc<Envelope> {
    /// Used by test suite to check round-trip encoding of `Envelope`.
    fn check_encoding(self) -> Result<Self, anyhow::Error> {
        let cbor = self.tagged_cbor();
        let restored = Envelope::from_tagged_cbor(&cbor);
        let restored = restored.map_err(|_| {
            println!("=== EXPECTED");
            println!("{}", self.format());
            println!("=== GOT");
            println!("{}", cbor.diagnostic());
            println!("===");
            anyhow!("Invalid format")
        })?;
        if self.digest() != restored.digest() {
            println!("=== EXPECTED");
            println!("{}", self.format());
            println!("=== GOT");
            println!("{}", restored.format());
            println!("===");
            bail!("Digest mismatch");
        }
        Ok(self)
    }
}
