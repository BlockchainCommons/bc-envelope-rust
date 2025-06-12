#![cfg(all(feature = "signature", feature = "recipient"))]

//! # Envelope Sealing and Unsealing
//!
//! This module provides convenience functions for combining signing and
//! encryption operations in a single step, creating secure, authenticated
//! envelopes.
//!
//! ## Sealing
//!
//! Sealing an envelope:
//! 1. Signs the envelope with the sender's private key
//! 2. Encrypts the signed envelope to the recipient's public key
//!
//! This creates a secure container where:
//! - The recipient can verify who sent the envelope (authentication)
//! - Only the intended recipient can decrypt the content (confidentiality)
//! - The signature ensures the content hasn't been modified (integrity)
//!
//! ## Unsealing
//!
//! Unsealing performs these operations in reverse:
//! 1. Decrypts the envelope using the recipient's private key
//! 2. Verifies the signature using the sender's public key

use anyhow::Result;
use bc_components::{Decrypter, Encrypter, Signer, SigningOptions, Verifier};

use crate::Envelope;

impl Envelope {
    /// Seals an envelope by signing it with the sender's key and then
    /// encrypting it to the recipient.
    ///
    /// This is a convenience method that combines signing and encryption in one
    /// step.
    ///
    /// # Arguments
    ///
    /// * `sender` - The private key used to sign the envelope
    /// * `recipient` - The public key used to encrypt the envelope
    ///
    /// # Returns
    ///
    /// A new envelope that has been signed and encrypted
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(all(feature = "signature", feature = "recipient"))]
    /// # {
    /// use bc_components::{EncapsulationScheme, SignatureScheme};
    /// use bc_envelope::prelude::*;
    ///
    /// // Create an envelope
    /// let envelope = Envelope::new("Confidential message");
    ///
    /// // Generate keys for sender and recipient using specific schemes
    /// let (sender_private, _) = SignatureScheme::Ed25519.keypair();
    /// let (_, recipient_public) = EncapsulationScheme::X25519.keypair();
    ///
    /// // Seal the envelope
    /// let sealed_envelope = envelope.seal(&sender_private, &recipient_public);
    /// # }
    /// ```
    pub fn seal(
        &self,
        sender: &dyn Signer,
        recipient: &dyn Encrypter,
    ) -> Envelope {
        self.sign(sender).encrypt_to_recipient(recipient)
    }

    /// Seals an envelope by signing it with the sender's key and then
    /// encrypting it to the recipient, with optional signing options.
    ///
    /// This is a convenience method that combines signing and encryption in one
    /// step.
    ///
    /// # Arguments
    ///
    /// * `sender` - The private key used to sign the envelope
    /// * `recipient` - The public key used to encrypt the envelope
    /// * `options` - Optional signing options to control how the signature is
    ///   created
    ///
    /// # Returns
    ///
    /// A new envelope that has been signed and encrypted
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(all(feature = "signature", feature = "recipient"))]
    /// # fn main() -> Result<(), anyhow::Error> {
    /// use bc_components::{EncapsulationScheme, SignatureScheme, SigningOptions};
    /// use bc_envelope::prelude::*;
    ///
    /// // Create an envelope
    /// let envelope = Envelope::new("Confidential message");
    ///
    /// // Generate keys for sender and recipient using specific schemes
    /// let (sender_private, _) = SignatureScheme::Ed25519.keypair();
    /// let (_, recipient_public) = EncapsulationScheme::X25519.keypair();
    ///
    /// // Create signing options for SSH
    /// let options = SigningOptions::Ssh {
    ///     namespace: "test".to_string(),
    ///     hash_alg: ssh_key::HashAlg::Sha512,
    /// };
    ///
    /// // Seal the envelope with options
    /// let sealed_envelope =
    ///     envelope.seal_opt(&sender_private, &recipient_public, Some(options));
    /// # Ok(())
    /// # }
    /// ```
    pub fn seal_opt(
        &self,
        sender: &dyn Signer,
        recipient: &dyn Encrypter,
        options: Option<SigningOptions>,
    ) -> Envelope {
        self.sign_opt(sender, options)
            .encrypt_to_recipient(recipient)
    }

    /// Unseals an envelope by decrypting it with the recipient's private key
    /// and then verifying the signature using the sender's public key.
    ///
    /// This is a convenience method that combines decryption and signature
    /// verification in one step.
    ///
    /// # Arguments
    ///
    /// * `sender` - The public key used to verify the signature
    /// * `recipient` - The private key used to decrypt the envelope
    ///
    /// # Returns
    ///
    /// A Result containing the unsealed envelope if successful, or an error if
    /// decryption or signature verification fails
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(all(feature = "signature", feature = "recipient"))]
    /// # fn main() -> Result<(), anyhow::Error> {
    /// use bc_components::{EncapsulationScheme, SignatureScheme};
    /// use bc_envelope::prelude::*;
    ///
    /// // Create an envelope
    /// let envelope = Envelope::new("Confidential message");
    ///
    /// // Generate keys for sender and recipient using specific schemes
    /// let (sender_private, sender_public) = SignatureScheme::Ed25519.keypair();
    /// let (recipient_private, recipient_public) =
    ///     EncapsulationScheme::X25519.keypair();
    ///
    /// // Seal the envelope
    /// let sealed_envelope = envelope.seal(&sender_private, &recipient_public);
    ///
    /// // Unseal the envelope using the recipient's private key
    /// let unsealed_envelope =
    ///     sealed_envelope.unseal(&sender_public, &recipient_private)?;
    ///
    /// // Verify we got back the original message
    /// let message: String = unsealed_envelope.extract_subject()?;
    /// assert_eq!(message, "Confidential message");
    /// # Ok(())
    /// # }
    /// ```
    pub fn unseal(
        &self,
        sender: &dyn Verifier,
        recipient: &dyn Decrypter,
    ) -> Result<Envelope> {
        self.decrypt_to_recipient(recipient)?.verify(sender)
    }
}

#[cfg(all(test, feature = "signature", feature = "recipient"))]
mod tests {
    use anyhow::Result;
    use bc_components::{EncapsulationScheme, SignatureScheme, SigningOptions};

    use super::*;

    #[test]
    fn test_seal_and_unseal() -> Result<()> {
        // Create a test envelope

        let message = "Top secret message";
        let original_envelope = Envelope::new(message);

        // Generate keys for sender and recipient using established schemes
        let (sender_private, sender_public) =
            SignatureScheme::Ed25519.keypair();
        let (recipient_private, recipient_public) =
            EncapsulationScheme::X25519.keypair();

        // Step 1: Seal the envelope
        let sealed_envelope =
            original_envelope.seal(&sender_private, &recipient_public);

        // Verify the envelope is encrypted
        assert!(sealed_envelope.is_subject_encrypted());

        // Step 2: Unseal the envelope
        let unsealed_envelope =
            sealed_envelope.unseal(&sender_public, &recipient_private)?;

        // Verify we got back the original message
        let extracted_message: String = unsealed_envelope.extract_subject()?;
        assert_eq!(extracted_message, message);

        Ok(())
    }

    #[test]
    #[cfg(all(feature = "signature", feature = "recipient"))]
    fn test_seal_opt_with_options() -> Result<()> {
        // Create a test envelope
        let message = "Confidential data";
        let original_envelope = Envelope::new(message);

        // Generate keys for sender and recipient
        let (sender_private, sender_public) =
            SignatureScheme::Ed25519.keypair();
        let (recipient_private, recipient_public) =
            EncapsulationScheme::X25519.keypair();

        // Create signing options
        let options = SigningOptions::Ssh {
            namespace: "test".to_string(),
            hash_alg: ssh_key::HashAlg::Sha512,
        };

        // Seal the envelope with options
        let sealed_envelope = original_envelope.seal_opt(
            &sender_private,
            &recipient_public,
            Some(options),
        );

        // Verify the envelope is encrypted
        assert!(sealed_envelope.is_subject_encrypted());

        // Unseal the envelope
        let unsealed_envelope =
            sealed_envelope.unseal(&sender_public, &recipient_private)?;

        // Verify we got back the original message
        let extracted_message: String = unsealed_envelope.extract_subject()?;
        assert_eq!(extracted_message, message);

        Ok(())
    }
}
