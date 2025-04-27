//! Extension for adding salt to envelopes to prevent correlation.
//!
//! This module provides functionality for decorrelating envelopes by adding random salt.
//! Salt is added as an assertion with the known value predicate 'salt' and a random value.
//! When an envelope is elided, this salt ensures that the digest of the elided envelope
//! cannot be correlated with other elided envelopes containing the same information.
//!
//! Decorrelation is an important privacy feature that prevents third parties from
//! determining whether two elided envelopes originally contained the same information
//! by comparing their digests.
//!
//! # Examples
//!
//! ```
//! use bc_envelope::prelude::*;
//!
//! // Create a simple envelope
//! let envelope = Envelope::new("Hello");
//!
//! // Create a decorrelated version by adding salt
//! let salted = envelope.add_salt();
//!
//! // The salted envelope has a different digest than the original
//! assert_ne!(envelope.digest(), salted.digest());
//!
//! // Format shows that the salted envelope has a salt assertion
//! assert!(salted.format_flat().contains("'salt': Salt"));
//! ```

use std::ops::RangeInclusive;

use crate::Envelope;
#[cfg(feature = "known_value")]
use known_values;

use anyhow::Result;
use bc_components::Salt;
use bc_rand::{RandomNumberGenerator, SecureRandomNumberGenerator};
use dcbor::prelude::*;

/// Support for decorrelation of envelopes using salt.
impl Envelope {
    /// Adds a proportionally-sized salt assertion to decorrelate the envelope.
    ///
    /// This method adds random salt bytes as an assertion to the envelope. The size of the salt
    /// is proportional to the size of the envelope being salted:
    /// - For small envelopes: 8-16 bytes
    /// - For larger envelopes: 5-25% of the envelope's size
    ///
    /// Salt is added as an assertion with the predicate 'salt' (a known value) and an object
    /// containing random bytes. This changes the digest of the envelope while preserving its
    /// semantic content, making it impossible to correlate with other envelopes containing
    /// the same information.
    ///
    /// # Returns
    ///
    /// A new envelope with the salt assertion added.
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Create an envelope with personally identifiable information
    /// let alice = Envelope::new("Alice")
    ///     .add_assertion("email", "alice@example.com")
    ///     .add_assertion("ssn", "123-45-6789");
    ///
    /// // Create a second envelope with the same information
    /// let alice2 = Envelope::new("Alice")
    ///     .add_assertion("email", "alice@example.com")
    ///     .add_assertion("ssn", "123-45-6789");
    ///
    /// // The envelopes have the same digest because they contain the same information
    /// assert_eq!(alice.digest(), alice2.digest());
    ///
    /// // Add salt to both envelopes
    /// let alice_salted = alice.add_salt();
    /// let alice2_salted = alice2.add_salt();
    ///
    /// // Now the envelopes have different digests, preventing correlation
    /// assert_ne!(alice_salted.digest(), alice2_salted.digest());
    ///
    /// // When elided, the salted envelopes still can't be correlated
    /// assert_ne!(alice_salted.elide().digest(), alice2_salted.elide().digest());
    /// ```
    pub fn add_salt(&self) -> Self {
        let mut rng = SecureRandomNumberGenerator;
        self.add_salt_using(&mut rng)
    }

    /// Adds the given Salt as an assertion to the envelope.
    ///
    /// This method attaches a pre-existing Salt object as an assertion to the envelope,
    /// using the known value 'salt' as the predicate. This is useful when you need to
    /// control the specific salt content being added.
    ///
    /// # Parameters
    ///
    /// * `salt` - A Salt object containing random bytes
    ///
    /// # Returns
    ///
    /// A new envelope with the salt assertion added.
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    /// use bc_components::Salt;
    ///
    /// // Create a salt with specific bytes
    /// let salt = Salt::from_data(vec![1, 2, 3, 4, 5, 6, 7, 8]);
    ///
    /// // Add this specific salt to an envelope
    /// let envelope = Envelope::new("Hello");
    /// let salted = envelope.add_salt_instance(salt);
    ///
    /// // The envelope now contains the salt assertion
    /// assert!(salted.format().contains("'salt': Salt"));
    /// ```
    pub fn add_salt_instance(&self, salt: Salt) -> Self {
        self.add_assertion(known_values::SALT, salt)
    }

    /// Adds salt of a specific byte length to the envelope.
    ///
    /// This method adds salt of a specified number of bytes to decorrelate the envelope.
    /// It requires that the byte count be at least 8 bytes (64 bits) to ensure sufficient
    /// entropy for effective decorrelation.
    ///
    /// # Parameters
    ///
    /// * `count` - The exact number of salt bytes to add
    ///
    /// # Returns
    ///
    /// A Result containing the new envelope with salt added, or an error if the byte
    /// count is less than the minimum (8 bytes).
    ///
    /// # Errors
    ///
    /// Returns an error if the specified byte count is less than 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// let envelope = Envelope::new("Hello");
    ///
    /// // Add exactly 16 bytes of salt
    /// let salted = envelope.add_salt_with_len(16).unwrap();
    ///
    /// // Trying to add less than 8 bytes will result in an error
    /// assert!(envelope.add_salt_with_len(7).is_err());
    /// ```
    pub fn add_salt_with_len(&self, count: usize) -> Result<Self> {
        let mut rng = SecureRandomNumberGenerator;
        self.add_salt_with_len_using(count, &mut rng)
    }

    /// Adds salt of a specific byte length to the envelope using the provided random number generator.
    ///
    /// This is an internal method that enables testing with deterministic random number generators.
    /// It should not be used directly by most code.
    ///
    /// # Parameters
    ///
    /// * `count` - The exact number of salt bytes to add
    /// * `rng` - The random number generator to use
    ///
    /// # Returns
    ///
    /// A Result containing the new envelope with salt added, or an error if the byte
    /// count is less than the minimum (8 bytes).
    #[doc(hidden)]
    pub fn add_salt_with_len_using(&self, count: usize, rng: &mut impl RandomNumberGenerator) -> Result<Self> {
        let salt = Salt::new_with_len_using(count, rng)?;
        Ok(self.add_salt_instance(salt))
    }

    /// Adds salt with a byte length randomly chosen from the given range.
    ///
    /// This method adds salt with a length randomly selected from the specified range
    /// to decorrelate the envelope. This approach provides additional decorrelation by
    /// varying the size of the salt itself.
    ///
    /// # Parameters
    ///
    /// * `range` - The inclusive range of byte lengths to choose from
    ///
    /// # Returns
    ///
    /// A Result containing the new envelope with salt added, or an error if the minimum
    /// value of the range is less than 8 bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the minimum of the range is less than 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    /// use std::ops::RangeInclusive;
    ///
    /// let envelope = Envelope::new("Hello");
    ///
    /// // Add salt with a length randomly chosen between 16 and 32 bytes
    /// let salted = envelope.add_salt_in_range(16..=32).unwrap();
    ///
    /// // Trying to use a range with minimum less than 8 will result in an error
    /// assert!(envelope.add_salt_in_range(4..=16).is_err());
    /// ```
    pub fn add_salt_in_range(&self, range: RangeInclusive<usize>) -> Result<Self> {
        let mut rng = SecureRandomNumberGenerator;
        self.add_salt_in_range_using(&range, &mut rng)
    }

    /// Adds salt with a byte length randomly chosen from the given range using the provided random number generator.
    ///
    /// This is an internal method that enables testing with deterministic random number generators.
    /// It should not be used directly by most code.
    ///
    /// # Parameters
    ///
    /// * `range` - The inclusive range of byte lengths to choose from
    /// * `rng` - The random number generator to use
    ///
    /// # Returns
    ///
    /// A Result containing the new envelope with salt added, or an error if the minimum
    /// value of the range is less than 8 bytes.
    #[doc(hidden)]
    pub fn add_salt_in_range_using(&self, range: &RangeInclusive<usize>, rng: &mut impl RandomNumberGenerator) -> Result<Self> {
        Ok(self.add_salt_instance(Salt::new_in_range_using(range, rng)?))
    }

    /// Adds salt with a size proportional to the envelope's size using the provided random number generator.
    ///
    /// This method is primarily for testing and internal use. It creates salt with a size
    /// proportional to the serialized size of the envelope and adds it as an assertion.
    ///
    /// # Parameters
    ///
    /// * `rng` - The random number generator to use
    ///
    /// # Returns
    ///
    /// A new envelope with the proportionally-sized salt assertion added.
    #[doc(hidden)]
    pub fn add_salt_using(&self, rng: &mut impl RandomNumberGenerator) -> Self {
        let salt = Salt::new_for_size_using(self.tagged_cbor().to_cbor_data().len(), rng);
        self.add_salt_instance(salt)
    }
}
