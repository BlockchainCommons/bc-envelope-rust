use std::rc::Rc;

use bc_components::DigestProvider;
use dcbor::{CBORTaggedEncodable, CBORError, CBORTaggedDecodable};

use crate::Envelope;

impl Envelope {
    /// Used by test suite to check round-trip encoding of `Envelope`.
    ///
    /// Not needed in production code.
    pub fn check_encoding(self: Rc<Envelope>) -> Result<Rc<Self>, CBORError> {
        let cbor = self.tagged_cbor();
        let restored = Envelope::from_tagged_cbor(&cbor);
        let restored = restored.map_err(|_| {
            println!("=== EXPECTED");
            println!("{}", self.format());
            println!("=== GOT");
            println!("{}", cbor.diagnostic());
            println!("===");
            CBORError::InvalidFormat
        })?;
        if self.digest() != restored.digest() {
            println!("=== EXPECTED");
            println!("{}", self.format());
            println!("=== GOT");
            println!("{}", restored.format());
            println!("===");
            return Err(CBORError::InvalidFormat);
        }
        Ok(self)
    }
}
