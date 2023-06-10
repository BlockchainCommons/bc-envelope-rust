use std::{rc::Rc, ops::RangeInclusive};

use crate::{Envelope, known_value_registry, Enclosable, Error};

use bc_components::Salt;
use bc_crypto::{RandomNumberGenerator, SecureRandomNumberGenerator};
use dcbor::{CBORTaggedEncodable, CBOREncodable};

/*```swift
public extension Envelope {
    /// Add the given Salt as an assertion
    func addSalt(_ salt: Salt) -> Envelope {
        addAssertion(.salt, salt)
    }

    /// Add a specified number of bytes of salt.
    func addSalt(_ count: Int) throws -> Envelope {
        guard let salt = Salt(count: count) else {
            throw EnvelopeError.invalidFormat
        }
        return addSalt(salt)
    }

    /// Add a number of bytes of salt chosen randomly from the given range.
    func addSalt(_ range: ClosedRange<Int>) throws -> Envelope {
        guard let salt = Salt(range: range) else {
            throw EnvelopeError.invalidFormat
        }
        return addSalt(salt)
    }

    /// Add a number of bytes of salt generally proportionate to the size of the object being salted.
    func addSalt() -> Envelope {
        var rng = SecureRandomNumberGenerator.shared
        return addSalt(using: &rng)
    }

    /// Add a deterministic amount of salt.
    ///
    /// Only used for testing.
    func addSalt<R: RandomNumberGenerator>(using rng: inout R) -> Envelope {
        addSalt(Salt(forSize: taggedCBOR.cborData.count, using: &rng))
    }
}
``` */

impl Envelope {
    /// Add a number of bytes of salt generally proportionate to the size of the object being salted.
    pub fn add_salt(self: Rc<Self>) -> Rc<Self> {
        let mut rng = SecureRandomNumberGenerator;
        self.add_salt_using(&mut rng)
    }

    /// Add the given Salt as an assertion
    pub fn add_salt_instance(self: Rc<Self>, salt: Salt) -> Rc<Self> {
        self.add_assertion(known_value_registry::SALT.enclose(), salt.enclose())
    }

    /// Add a specified number of bytes of salt.
    ///
    /// Returns an error if the number of bytes is less than 8.
    pub fn add_salt_with_len(self: Rc<Self>, count: usize) -> Result<Rc<Self>, Error> {
        let mut rng = SecureRandomNumberGenerator;
        self.add_salt_with_len_using(count, &mut rng)
    }

    /// Add a specified number of bytes of salt.
    ///
    /// Returns an error if the number of bytes is less than 8.
    pub fn add_salt_with_len_using(self: Rc<Self>, count: usize, rng: &mut impl RandomNumberGenerator) -> Result<Rc<Self>, Error> {
        let salt = Salt::new_with_len_using(count, rng).ok_or(Error::InvalidFormat)?;
        Ok(self.add_salt_instance(salt))
    }

    /// Add a number of bytes of salt chosen randomly from the given range.
    ///
    /// Returns an error if the minimum number of bytes is less than 8.
    pub fn add_salt_in_range(self: Rc<Self>, range: RangeInclusive<usize>) -> Result<Rc<Self>, Error> {
        let mut rng = SecureRandomNumberGenerator;
        self.add_salt_in_range_using(&range, &mut rng)
    }

    /// Add a number of bytes of salt chosen randomly from the given range.
    ///
    /// Returns an error if the minimum number of bytes is less than 8.
    pub fn add_salt_in_range_using(self: Rc<Self>, range: &RangeInclusive<usize>, rng: &mut impl RandomNumberGenerator) -> Result<Rc<Self>, Error> {
        let salt = Salt::new_in_range_using(range, rng).ok_or(Error::InvalidFormat)?;
        Ok(self.add_salt_instance(salt))
    }

    /// Add a deterministic amount of salt.
    ///
    /// Only used for testing.
    pub fn add_salt_using(self: Rc<Self>, rng: &mut impl RandomNumberGenerator) -> Rc<Self> {
        let salt = Salt::new_for_size_using(self.tagged_cbor().cbor_data().len(), rng);
        self.add_salt_instance(salt)
    }
}
