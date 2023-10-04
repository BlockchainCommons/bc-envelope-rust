use std::{rc::Rc, ops::RangeInclusive};

use crate::{Envelope, extension::known_values};

use bc_components::Salt;
use bc_rand::{RandomNumberGenerator, SecureRandomNumberGenerator};
use dcbor::prelude::*;

/// Support for decorrelation of envelopes using salt.
impl Envelope {
    /// Add a number of bytes of salt generally proportionate to the size of the object being salted.
    pub fn add_salt(self: Rc<Self>) -> Rc<Self> {
        let mut rng = SecureRandomNumberGenerator;
        self.add_salt_using(&mut rng)
    }

    /// Add the given Salt as an assertion
    pub fn add_salt_instance(self: Rc<Self>, salt: Salt) -> Rc<Self> {
        self.add_assertion(known_values::SALT, salt)
    }

    /// Add a specified number of bytes of salt.
    ///
    /// Returns an error if the number of bytes is less than 8.
    pub fn add_salt_with_len(self: Rc<Self>, count: usize) -> anyhow::Result<Rc<Self>> {
        let mut rng = SecureRandomNumberGenerator;
        self.add_salt_with_len_using(count, &mut rng)
    }

    #[doc(hidden)]
    /// Add a specified number of bytes of salt.
    ///
    /// Returns an error if the number of bytes is less than 8.
    pub fn add_salt_with_len_using(self: Rc<Self>, count: usize, rng: &mut impl RandomNumberGenerator) -> anyhow::Result<Rc<Self>> {
        let salt = Salt::new_with_len_using(count, rng)?;
        Ok(self.add_salt_instance(salt))
    }

    /// Add a number of bytes of salt chosen randomly from the given range.
    ///
    /// Returns an error if the minimum number of bytes is less than 8.
    pub fn add_salt_in_range(self: Rc<Self>, range: RangeInclusive<usize>) -> anyhow::Result<Rc<Self>> {
        let mut rng = SecureRandomNumberGenerator;
        self.add_salt_in_range_using(&range, &mut rng)
    }

    #[doc(hidden)]
    /// Add a number of bytes of salt chosen randomly from the given range.
    ///
    /// Returns an error if the minimum number of bytes is less than 8.
    pub fn add_salt_in_range_using(self: Rc<Self>, range: &RangeInclusive<usize>, rng: &mut impl RandomNumberGenerator) -> anyhow::Result<Rc<Self>> {
        Ok(self.add_salt_instance(Salt::new_in_range_using(range, rng)?))
    }

    #[doc(hidden)]
    /// Add a deterministic amount of salt.
    ///
    /// Only used for testing.
    pub fn add_salt_using(self: Rc<Self>, rng: &mut impl RandomNumberGenerator) -> Rc<Self> {
        let salt = Salt::new_for_size_using(self.tagged_cbor().cbor_data().len(), rng);
        self.add_salt_instance(salt)
    }
}
