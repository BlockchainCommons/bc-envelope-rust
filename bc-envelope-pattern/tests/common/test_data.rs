#![allow(dead_code)]

use std::collections::HashSet;

use bc_components::{
    Nonce, PrivateKeyBase, PublicKeys, PublicKeysProvider, SymmetricKey,
};
use bc_envelope::prelude::*;
use hex_literal::hex;

pub const PLAINTEXT_HELLO: &str = "Hello.";

pub fn hello_envelope() -> Envelope { Envelope::new(PLAINTEXT_HELLO) }
#[cfg(feature = "known_value")]
pub fn known_value_envelope() -> Envelope { Envelope::new(known_values::NOTE) }
pub fn assertion_envelope() -> Envelope {
    Envelope::new_assertion("knows", "Bob")
}

pub fn single_assertion_envelope() -> Envelope {
    Envelope::new("Alice").add_assertion("knows", "Bob")
}

pub fn double_assertion_envelope() -> Envelope {
    single_assertion_envelope().add_assertion("knows", "Carol")
}

pub fn wrapped_envelope() -> Envelope { hello_envelope().wrap_envelope() }
pub fn double_wrapped_envelope() -> Envelope {
    wrapped_envelope().wrap_envelope()
}

pub fn alice_seed() -> Vec<u8> {
    hex!("82f32c855d3d542256180810797e0073").into()
}
pub fn alice_private_key() -> PrivateKeyBase {
    PrivateKeyBase::from_data(alice_seed())
}
pub fn alice_public_key() -> PublicKeys { alice_private_key().public_keys() }

// pub fn bob_identifier() -> ARID {
// ARID::from_data(hex!("
// d44c5e0afd353f47b02f58a5a3a29d9a2efa6298692f896cd2923268599a0d0f")) }
pub fn bob_seed() -> Vec<u8> { hex!("187a5973c64d359c836eba466a44db7b").into() }
pub fn bob_private_key() -> PrivateKeyBase {
    PrivateKeyBase::from_data(bob_seed())
}
pub fn bob_public_key() -> PublicKeys { bob_private_key().public_keys() }

// pub fn carol_identifier() -> ARID {
// ARID::from_data(hex!("
// 06c777262faedf49a443277474c1c08531efcff4c58e9cb3b04f7fc1c0e6d60d")) }
pub fn carol_seed() -> Vec<u8> {
    hex!("8574afab18e229651c1be8f76ffee523").into()
}
pub fn carol_private_key() -> PrivateKeyBase {
    PrivateKeyBase::from_data(carol_seed())
}
pub fn carol_public_key() -> PublicKeys { carol_private_key().public_keys() }

// pub fn example_ledger_identifier() -> ARID {
// ARID::from_data(hex!("
// 0eda5ce79a2b5619e387f490861a2e7211559029b3b369cf98fb749bd3ba9a5d")) }
// pub fn example_ledger_seed() -> Vec<u8> {
// hex!("d6737ab34e4e8bb05b6ac035f9fba81a").into() }
// pub fn example_ledger_private_key() -> PrivateKeyBase {
// PrivateKeyBase::from_data(&example_ledger_seed()) }
// pub fn example_ledger_public_key() -> PublicKeys {
// example_ledger_private_key().public_key() }

// pub fn state_identifier() -> ARID {
// ARID::from_data(hex!("
// 04363d5ff99733bc0f1577baba440af1cf344ad9e454fad9d128c00fef6505e8")) }
// pub fn state_seed() -> Vec<u8> {
// hex!("3e9271f46cdb85a3b584e7220b976918").into() } pub fn state_private_key()
// -> PrivateKeyBase { PrivateKeyBase::from_data(&state_seed()) }
// pub fn state_public_key() -> PublicKeys { state_private_key().public_key() }

pub fn fake_content_key() -> SymmetricKey {
    SymmetricKey::from_data(hex!(
        "526afd95b2229c5381baec4a1788507a3c4a566ca5cce64543b46ad12aff0035"
    ))
}
pub fn fake_nonce() -> Nonce {
    Nonce::from_data(hex!("4d785658f36c22fb5aed3ac0"))
}

#[cfg(feature = "signature")]
pub fn credential() -> Envelope {
    use std::{cell::RefCell, rc::Rc};

    use bc_components::ARID;
    use bc_envelope::SigningOptions;
    use bc_rand::make_fake_random_number_generator;

    use crate::common::check_encoding::CheckEncoding;

    let rng = Rc::new(RefCell::new(make_fake_random_number_generator()));
    let options = SigningOptions::Schnorr { rng };
    Envelope::new(ARID::from_data(hex!(
        "4676635a6e6068c2ef3ffd8ff726dd401fd341036e920f136a1d8af5e829496d"
    )))
    .add_assertion(known_values::IS_A, "Certificate of Completion")
    .add_assertion(known_values::ISSUER, "Example Electrical Engineering Board")
    .add_assertion(
        known_values::CONTROLLER,
        "Example Electrical Engineering Board",
    )
    .add_assertion("firstName", "James")
    .add_assertion("lastName", "Maxwell")
    .add_assertion("issueDate", dcbor::Date::from_string("2020-01-01").unwrap())
    .add_assertion(
        "expirationDate",
        dcbor::Date::from_string("2028-01-01").unwrap(),
    )
    .add_assertion("photo", "This is James Maxwell's photo.")
    .add_assertion("certificateNumber", "123-456-789")
    .add_assertion("subject", "RF and Microwave Engineering")
    .add_assertion("continuingEducationUnits", 1)
    .add_assertion("professionalDevelopmentHours", 15)
    .add_assertion("topics", vec!["Subject 1", "Subject 2"].to_cbor())
    .wrap_envelope()
    .add_signature_opt(&alice_private_key(), Some(options), None)
    .add_assertion(
        known_values::NOTE,
        "Signed by Example Electrical Engineering Board",
    )
    .check_encoding()
    .unwrap()
}

pub fn redacted_credential() -> Envelope {
    let credential = credential();
    let mut target = HashSet::new();
    target.insert(credential.digest().into_owned());
    for assertion in credential.assertions() {
        target.extend(assertion.deep_digests());
    }
    target.insert(credential.subject().digest().into_owned());
    let content = credential.subject().unwrap_envelope().unwrap();
    target.insert(content.digest().into_owned());
    target.insert(content.subject().digest().into_owned());

    target.extend(
        content
            .assertion_with_predicate("firstName")
            .unwrap()
            .shallow_digests(),
    );
    target.extend(
        content
            .assertion_with_predicate("lastName")
            .unwrap()
            .shallow_digests(),
    );
    target.extend(
        content
            .assertion_with_predicate(known_values::IS_A)
            .unwrap()
            .shallow_digests(),
    );
    target.extend(
        content
            .assertion_with_predicate(known_values::ISSUER)
            .unwrap()
            .shallow_digests(),
    );
    target.extend(
        content
            .assertion_with_predicate("subject")
            .unwrap()
            .shallow_digests(),
    );
    target.extend(
        content
            .assertion_with_predicate("expirationDate")
            .unwrap()
            .shallow_digests(),
    );
    credential.elide_revealing_set(&target)
}
