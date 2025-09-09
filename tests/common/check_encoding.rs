use bc_components::DigestProvider;
use dcbor::prelude::*;

use bc_envelope::{Envelope, Error, Result};

pub trait CheckEncoding {
    #[allow(dead_code)]
    fn check_encoding(&self) -> Result<Self>
    where
        Self: Sized;
}

impl CheckEncoding for Envelope {
    /// Used by test suite to check round-trip encoding of `Envelope`.
    fn check_encoding(&self) -> Result<Self> {
        let cbor = self.tagged_cbor();
        let restored = Envelope::from_tagged_cbor(cbor.clone());
        let restored = restored.map_err(|_| {
            println!("=== EXPECTED");
            println!("{}", self.format());
            println!("=== GOT");
            println!("{}", cbor.diagnostic());
            println!("===");
            Error::msg("Invalid format")
        })?;
        if self.digest() != restored.digest() {
            println!("=== EXPECTED");
            println!("{}", self.format());
            println!("=== GOT");
            println!("{}", restored.format());
            println!("===");
            return Err(Error::msg("Digest mismatch"));
        }
        Ok(self.clone())
    }
}
