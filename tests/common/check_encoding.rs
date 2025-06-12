use anyhow::{anyhow, bail};
use bc_components::DigestProvider;
use dcbor::prelude::*;

use crate::Envelope;

pub trait CheckEncoding {
    #[allow(dead_code)]
    fn check_encoding(&self) -> anyhow::Result<Self>
    where
        Self: Sized;
}

impl CheckEncoding for Envelope {
    /// Used by test suite to check round-trip encoding of `Envelope`.
    fn check_encoding(&self) -> anyhow::Result<Self> {
        let cbor = self.tagged_cbor();
        let restored = Envelope::from_tagged_cbor(cbor.clone());
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
        Ok(self.clone())
    }
}
