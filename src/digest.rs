use bc_components::{Digest, DigestProvider};

use crate::envelope::Envelope;

/// Support for calculating the digests associated with `Envelope`.

impl DigestProvider for Envelope {
    /// The envelope's digest.
    ///
    /// This digest can be used to compare two envelopes for semantic equivalence, that
    /// is, the two envelopes would contain the same information in their unencrypted
    /// and unelided forms. See <doc:Diffing> for more information.
    fn digest(&self) -> Digest {
        match self {
            Envelope::Node {
                subject: _,
                assertions: _,
                digest,
            } => digest.clone(),
            Envelope::Leaf { cbor: _, digest } => digest.clone(),
            Envelope::Wrapped { envelope: _, digest } => digest.clone(),
            Envelope::KnownValue { value: _, digest } => digest.clone(),
            Envelope::Assertion(assertion) => assertion.digest(),
            Envelope::Encrypted(encrypted_message) => encrypted_message.digest(),
            Envelope::Compressed(compressed) => compressed.digest(),
            Envelope::Elided(digest) => digest.clone(),
        }
    }
}
