use std::collections::HashMap;

use anyhow::{Result, bail};
pub use bc_components::{
    SSKRError, SSKRGroupSpec, SSKRSecret, SSKRShare, SSKRSpec,
};
use bc_components::{SymmetricKey, sskr_combine, sskr_generate_using};
use bc_rand::RandomNumberGenerator;
#[cfg(feature = "known_value")]
use known_values;

use crate::{Envelope, Error};

/// Support for splitting and combining envelopes using SSKR (Shamir's Secret
/// Sharing).
///
/// This module extends Gordian Envelope with functions to support Sharded
/// Secret Key Reconstruction (SSKR), which is an implementation of Shamir's
/// Secret Sharing. SSKR allows splitting a secret (in this case, a symmetric
/// encryption key) into multiple shares, with a threshold required for
/// reconstruction.
///
/// SSKR provides social recovery for encrypted envelopes by allowing the owner
/// to distribute shares to trusted individuals or storage locations, with a
/// specified threshold required to reconstruct the original envelope.
///
/// # How SSKR Works with Envelopes
///
/// The overall process is as follows:
///
/// 1. A Gordian Envelope is encrypted with a symmetric key
/// 2. The symmetric key is split into multiple SSKR shares using a group
///    threshold and member thresholds
/// 3. Each share is added as an assertion to a copy of the encrypted envelope
/// 4. These envelope copies are distributed to trusted individuals or storage
///    locations
/// 5. Later, when enough shares are brought together, the original envelope can
///    be reconstructed
///
/// Sharded Secret Key Reconstruction (SSKR) for Envelopes.
///
/// For a complete working example, see the `sskr_split()` and `sskr_join()`
/// method documentation.
impl Envelope {
    /// Returns a new `Envelope` with a `sskrShare: SSKRShare` assertion added.
    ///
    /// This is an internal helper function used by the SSKR split methods.
    fn add_sskr_share(&self, share: &SSKRShare) -> Self {
        self.add_assertion(known_values::SSKR_SHARE, share.clone())
    }

    /// Splits the envelope into a set of SSKR shares.
    ///
    /// This method splits the symmetric key used to encrypt the envelope into
    /// SSKR shares, and returns multiple copies of the original envelope,
    /// each with a different SSKR share added as an assertion. The envelope
    /// subject should already be encrypted with the provided `content_key`.
    ///
    /// The returned structure is a nested array that preserves the group
    /// structure of the SSKR shares. Each outer array represents a group,
    /// and each inner array contains the shares for that group.
    ///
    /// # Parameters
    ///
    /// * `spec` - The SSKR specification that defines the group structure and
    ///   thresholds
    /// * `content_key` - The symmetric key that was used to encrypt the
    ///   envelope
    ///
    /// # Returns
    ///
    /// A nested array of envelopes organized by groups, each envelope
    /// containing the original encrypted envelope with a unique SSKR share
    /// added as an assertion
    ///
    /// # Errors
    ///
    /// Returns an error if the SSKR shares cannot be generated
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    /// use bc_components::{SymmetricKey, SSKRGroupSpec, SSKRSpec};
    ///
    /// // Create and encrypt an envelope
    /// let envelope = Envelope::new("Secret data").encrypt(&SymmetricKey::new());
    ///
    /// // Define a 2-of-3 SSKR split
    /// let group_spec = SSKRGroupSpec::new(2, 3).unwrap();
    /// let spec = SSKRSpec::new(1, vec![group_spec]).unwrap();
    ///
    /// // Create a symmetric key for the encrypted content
    /// let content_key = SymmetricKey::new();
    ///
    /// // Split the envelope into shares
    /// let shares = envelope.sskr_split(&spec, &content_key).unwrap();
    ///
    /// // The outer array represents groups (in this case, a single group)
    /// assert_eq!(shares.len(), 1);
    ///
    /// // The inner array contains the shares for the group (3 shares in this example)
    /// assert_eq!(shares[0].len(), 3);
    ///
    /// // Each share is an envelope with an 'sskrShare' assertion
    /// for share in &shares[0] {
    ///     assert!(share.assertions_with_predicate(known_values::SSKR_SHARE).len() > 0);
    /// }
    /// ```
    pub fn sskr_split(
        &self,
        spec: &SSKRSpec,
        content_key: &SymmetricKey,
    ) -> Result<Vec<Vec<Envelope>>> {
        let mut rng = bc_rand::SecureRandomNumberGenerator;
        self.sskr_split_using(spec, content_key, &mut rng)
    }

    /// Splits the envelope into a flattened set of SSKR shares.
    ///
    /// This method works like `sskr_split()` but returns a flat array of all
    /// shares rather than preserving the group structure. This is
    /// convenient when the group structure is not needed for distribution.
    ///
    /// # Parameters
    ///
    /// * `spec` - The SSKR specification that defines the group structure and
    ///   thresholds
    /// * `content_key` - The symmetric key that was used to encrypt the
    ///   envelope
    ///
    /// # Returns
    ///
    /// A flat array of all envelopes containing SSKR shares
    ///
    /// # Errors
    ///
    /// Returns an error if the SSKR shares cannot be generated
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_components::{SSKRGroupSpec, SSKRSpec, SymmetricKey};
    /// use bc_envelope::prelude::*;
    ///
    /// // Create and encrypt an envelope
    /// let envelope = Envelope::new("Secret data").encrypt(&SymmetricKey::new());
    ///
    /// // Define a 2-of-3 SSKR split in a single group
    /// let group_spec = SSKRGroupSpec::new(2, 3).unwrap();
    /// let spec = SSKRSpec::new(1, vec![group_spec]).unwrap();
    ///
    /// // Create a symmetric key for the encrypted content
    /// let content_key = SymmetricKey::new();
    ///
    /// // Split the envelope into a flat array of shares
    /// let shares = envelope.sskr_split_flattened(&spec, &content_key).unwrap();
    ///
    /// // We get all 3 shares in a flat array
    /// assert_eq!(shares.len(), 3);
    ///
    /// // Each share is an envelope with an 'sskrShare' assertion
    /// for share in &shares {
    ///     assert!(
    ///         share
    ///             .assertions_with_predicate(known_values::SSKR_SHARE)
    ///             .len()
    ///             > 0
    ///     );
    /// }
    /// ```
    pub fn sskr_split_flattened(
        &self,
        spec: &SSKRSpec,
        content_key: &SymmetricKey,
    ) -> Result<Vec<Envelope>> {
        Ok(self
            .sskr_split(spec, content_key)?
            .into_iter()
            .flatten()
            .collect())
    }

    #[doc(hidden)]
    /// Internal function that splits the envelope using a provided random
    /// number generator.
    ///
    /// This method is primarily used for testing to ensure deterministic SSKR
    /// shares.
    pub fn sskr_split_using(
        &self,
        spec: &SSKRSpec,
        content_key: &SymmetricKey,
        test_rng: &mut impl RandomNumberGenerator,
    ) -> Result<Vec<Vec<Envelope>>> {
        let master_secret = SSKRSecret::new(content_key.data())?;
        let shares = sskr_generate_using(spec, &master_secret, test_rng)?;
        let mut result: Vec<Vec<Envelope>> = Vec::new();
        for group in shares {
            let mut group_result: Vec<Envelope> = Vec::new();
            for share in group {
                let share_result = self.add_sskr_share(&share);
                group_result.push(share_result);
            }
            result.push(group_result);
        }
        Ok(result)
    }

    /// Internal helper function that extracts SSKR shares from a set of
    /// envelopes.
    ///
    /// This function groups the shares by their identifier (which must match
    /// for shares to be combined).
    fn sskr_shares_in(
        envelopes: &[&Envelope],
    ) -> Result<HashMap<u16, Vec<SSKRShare>>> {
        let mut result: HashMap<u16, Vec<SSKRShare>> = HashMap::new();
        for envelope in envelopes {
            for assertion in
                envelope.assertions_with_predicate(known_values::SSKR_SHARE)
            {
                let share = assertion
                    .as_object()
                    .unwrap()
                    .extract_subject::<SSKRShare>()?;
                let identifier = share.identifier();
                result
                    .entry(identifier)
                    .and_modify(|shares| shares.push(share.clone()))
                    .or_insert(vec![share]);
            }
        }
        Ok(result)
    }

    /// Reconstructs the original envelope from a set of SSKR shares.
    ///
    /// Given a set of envelopes with SSKR share assertions, this method
    /// attempts to combine the shares to reconstruct the original symmetric
    /// key. If successful, it uses the key to decrypt the envelope and
    /// return the original envelope subject.
    ///
    /// The method will try all combinations of shares with matching identifiers
    /// to find a valid reconstruction.
    ///
    /// # Parameters
    ///
    /// * `envelopes` - An array of envelope references containing SSKR shares
    ///
    /// # Returns
    ///
    /// The original envelope if reconstruction is successful
    ///
    /// # Errors
    ///
    /// * Returns `EnvelopeError::InvalidShares` if not enough valid shares are
    ///   provided
    /// * Returns various errors if decryption fails
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    /// use bc_components::{SymmetricKey, SSKRGroupSpec, SSKRSpec};
    ///
    /// // Create the original envelope with an assertion
    /// let original = Envelope::new("Secret message")
    ///     .add_assertion("metadata", "This is a test");
    ///
    /// // Create a content key
    /// let content_key = SymmetricKey::new();
    ///
    /// // Wrap the envelope (so the whole envelope including its assertions
    /// // become the subject)
    /// let wrapped_original = original
    ///     .wrap();
    ///
    /// // Encrypt the wrapped envelope
    /// let encrypted = wrapped_original
    ///     .encrypt_subject(&content_key).unwrap();
    ///
    /// // Create a 2-of-3 SSKR split specification
    /// let group = SSKRGroupSpec::new(2, 3).unwrap();
    /// let spec = SSKRSpec::new(1, vec![group]).unwrap();
    ///
    /// // Split the encrypted envelope into shares
    /// let shares = encrypted.sskr_split(&spec, &content_key).unwrap();
    /// assert_eq!(shares[0].len(), 3);
    ///
    /// // The shares would normally be distributed to different people/places
    /// // For recovery, we need at least the threshold number of shares (2 in this case)
    /// let share1 = &shares[0][0];
    /// let share2 = &shares[0][1];
    ///
    /// // Combine the shares to recover the original decrypted envelope
    /// let recovered_wrapped = Envelope::sskr_join(&[share1, share2]).unwrap();
    ///
    /// // Unwrap the envelope to get the original envelope
    /// let recovered = recovered_wrapped.try_unwrap().unwrap();
    ///
    /// // Check that the recovered envelope matches the original
    /// assert!(recovered.is_identical_to(&original));
    /// ```
    pub fn sskr_join(envelopes: &[&Envelope]) -> Result<Envelope> {
        if envelopes.is_empty() {
            bail!(Error::InvalidShares);
        }

        let grouped_shares: Vec<_> =
            Self::sskr_shares_in(envelopes)?.values().cloned().collect();
        for shares in grouped_shares {
            if let Ok(secret) = sskr_combine(&shares) {
                if let Ok(content_key) = SymmetricKey::from_data_ref(&secret) {
                    if let Ok(envelope) =
                        envelopes.first().unwrap().decrypt_subject(&content_key)
                    {
                        return Ok(envelope.subject());
                    }
                }
            }
        }
        bail!(Error::InvalidShares)
    }
}

#[cfg(all(test, feature = "sskr", feature = "types", feature = "known_value"))]
mod tests {
    use bc_components::{SSKRGroupSpec, SSKRSpec, SymmetricKey};

    use crate::prelude::*;

    #[test]
    fn test_sskr_split_and_join() {
        // Create the original envelope with an assertion
        let original = Envelope::new("Secret message")
            .add_assertion("metadata", "This is a test");

        // Create a content key
        let content_key = SymmetricKey::new();

        // Wrap the envelope (so the whole envelope including its assertions
        // become the subject)
        let wrapped_original = original.wrap();

        // Encrypt the wrapped envelope
        let encrypted = wrapped_original.encrypt_subject(&content_key).unwrap();

        // Create a 2-of-3 SSKR split specification
        let group = SSKRGroupSpec::new(2, 3).unwrap();
        let spec = SSKRSpec::new(1, vec![group]).unwrap();

        // Split the encrypted envelope into shares
        let shares = encrypted.sskr_split(&spec, &content_key).unwrap();
        assert_eq!(shares[0].len(), 3);

        // The shares would normally be distributed to different people/places
        // For recovery, we need at least the threshold number of shares (2 in
        // this case)
        let share1 = &shares[0][0];
        let share2 = &shares[0][1];

        // Combine the shares to recover the original decrypted envelope
        let recovered_wrapped = Envelope::sskr_join(&[share1, share2]).unwrap();

        // Unwrap the envelope to get the original envelope
        let recovered = recovered_wrapped.try_unwrap().unwrap();

        // Check that the recovered envelope matches the original
        assert!(recovered.is_identical_to(&original));
    }
}
