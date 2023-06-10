use std::{rc::Rc, collections::HashMap};

pub use bc_components::{SSKRShare, SSKRSpec, SSKRGroupSpec, SymmetricKey, SSKRSecret};
use bc_components::{sskr_generate_using, sskr_combine};
use bc_crypto::RandomNumberGenerator;

use crate::{Envelope, known_value_registry, Error};

impl Envelope {
    /// Returns a new ``Envelope`` with a `sskrShare: SSKRShare` assertion added.
    fn add_sskr_share(self: Rc<Self>, share: &SSKRShare) -> Rc<Self> {
        self.add_assertion(known_value_registry::SSKR_SHARE, share)
    }

    /// Splits the envelope into a set of SSKR shares.
    ///
    /// The envelope subject should already be encrypted by a specific `SymmetricKey`
    /// known as the `contentKey`.
    ///
    /// Each returned envelope will have an `sskrShare: SSKRShare` assertion added to
    /// it.
    ///
    /// - Parameters:
    ///   - spec: The SSKR split specification.
    ///   - contentKey: The `SymmetricKey` used to encrypt the envelope's subject.
    ///
    /// - Returns: An array of arrays. Each element of the outer array represents an
    /// SSKR group, and the elements of each inner array are the envelope with a unique
    /// `sskrShare: SSKRShare` assertion added to each.
    pub fn sskr_split(self: Rc<Self>, spec: &SSKRSpec, content_key: &SymmetricKey) -> Result<Vec<Vec<Rc<Envelope>>>, Error> {
        let mut rng = bc_crypto::SecureRandomNumberGenerator;
        self.sskr_split_using(spec, content_key, &mut rng)
    }

    /// Splits the envelope into a set of SSKR shares.
    ///
    /// The envelope subject should already be encrypted by a specific `SymmetricKey`
    /// known as the `contentKey`.
    ///
    /// Each returned envelope will have an `sskrShare: SSKRShare` assertion added to
    /// it.
    ///
    /// - Parameters:
    ///   - spec: The SSKR split specification.
    ///   - contentKey: The `SymmetricKey` used to encrypt the envelope's subject.
    ///
    /// - Returns: An array of arrays. Each element of the outer array represents an
    /// SSKR group, and the elements of each inner array are the envelope with a unique
    /// `sskrShare: SSKRShare` assertion added to each.
    pub fn sskr_split_using(self: Rc<Self>, spec: &SSKRSpec, content_key: &SymmetricKey, test_rng: &mut impl RandomNumberGenerator) -> Result<Vec<Vec<Rc<Envelope>>>, Error> {
        let master_secret = SSKRSecret::new(content_key)?;
        let shares = sskr_generate_using(spec, &master_secret, test_rng)?;
        let mut result: Vec<Vec<Rc<Envelope>>> = Vec::new();
        for group in shares {
            let mut group_result: Vec<Rc<Envelope>> = Vec::new();
            for share in group {
                let share_result = self.clone().add_sskr_share(&share);
                group_result.push(share_result);
            }
            result.push(group_result);
        }
        Ok(result)
    }
}

impl Envelope {
    pub fn sskr_shares_in(envelopes: &[Rc<Envelope>]) -> Result<HashMap<u16, Vec<SSKRShare>>, Error> {
        let mut result: HashMap<u16, Vec<SSKRShare>> = HashMap::new();
        for envelope in envelopes {
            for assertion in envelope.clone().assertions_with_predicate(known_value_registry::SSKR_SHARE) {
                let share = assertion.object().unwrap().extract_subject::<SSKRShare>()?;
                let identifier = share.clone().identifier();
                if result.get(&identifier).is_none() {
                    result.insert(identifier, Vec::new());
                }
                result.get_mut(&identifier).unwrap().push((*share).clone());
            }
        }
        Ok(result)
    }
}

impl Envelope {
    /// Creates a new envelope resulting from the joining a set of envelopes split by SSKR.
    ///
    /// Given a set of envelopes that are ostensibly all part of the same SSKR split,
    /// this method attempts to reconstuct the original envelope subject. It will try
    /// all present `sskrShare: SSKRShare` assertions, grouped by split ID, to achieve a
    /// threshold of shares. If it can do so successfully the initializer succeeeds.
    ///
    /// - Parameter envelopes: The envelopes to be joined.
    ///
    /// - Throws: Throws an exception if no quorum of shares can be found to reconstruct
    /// the original envelope.
    pub fn sskr_join(envelopes: &[Rc<Envelope>]) -> Result<Rc<Envelope>, Error> {
        if envelopes.is_empty() {
            return Err(Error::InvalidShares);
        }

        let grouped_shares: Vec<_> = Self::sskr_shares_in(envelopes)?.values().cloned().collect();
        for shares in grouped_shares {
            if let Ok(secret) = sskr_combine(&shares) {
                if let Some(content_key) = SymmetricKey::from_data_ref(&secret) {
                    if let Ok(envelope) = envelopes.first().unwrap().clone().decrypt_subject(&content_key) {
                        return Ok(envelope.subject());
                    }
                }
            }
        }
        Err(Error::InvalidShares)
    }
}