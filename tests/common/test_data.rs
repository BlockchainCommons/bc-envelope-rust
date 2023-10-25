#![allow(dead_code)]

use bc_envelope::prelude::*;
use hex_literal::hex;
use bc_components::{PrivateKeyBase, PublicKeyBase, SymmetricKey, Nonce};

pub const PLAINTEXT_HELLO: &str = "Hello.";

pub fn hello_envelope() -> Envelope { Envelope::new(PLAINTEXT_HELLO) }
#[cfg(feature = "known_value")]
pub fn known_value_envelope() -> Envelope { Envelope::new(known_values::NOTE) }
pub fn assertion_envelope() -> Envelope { Envelope::new_assertion("knows", "Bob") }

pub fn single_assertion_envelope() -> Envelope {
    Envelope::new("Alice")
        .add_assertion("knows", "Bob")
}

pub fn double_assertion_envelope() -> Envelope {
    single_assertion_envelope()
        .add_assertion("knows", "Carol")
}

pub fn wrapped_envelope() -> Envelope { hello_envelope().wrap_envelope() }
pub fn double_wrapped_envelope() -> Envelope { wrapped_envelope().wrap_envelope() }

pub fn alice_seed() -> Vec<u8> { hex!("82f32c855d3d542256180810797e0073").into() }
pub fn alice_private_keys() -> PrivateKeyBase { PrivateKeyBase::from_data(alice_seed()) }
pub fn alice_public_keys() -> PublicKeyBase { alice_private_keys().public_keys() }

// pub fn bob_identifier() -> ARID { ARID::from_data(hex!("d44c5e0afd353f47b02f58a5a3a29d9a2efa6298692f896cd2923268599a0d0f")) }
pub fn bob_seed() -> Vec<u8> { hex!("187a5973c64d359c836eba466a44db7b").into() }
pub fn bob_private_keys() -> PrivateKeyBase { PrivateKeyBase::from_data(bob_seed()) }
pub fn bob_public_keys() -> PublicKeyBase { bob_private_keys().public_keys() }

// pub fn carol_identifier() -> ARID { ARID::from_data(hex!("06c777262faedf49a443277474c1c08531efcff4c58e9cb3b04f7fc1c0e6d60d")) }
pub fn carol_seed() -> Vec<u8> { hex!("8574afab18e229651c1be8f76ffee523").into() }
pub fn carol_private_keys() -> PrivateKeyBase { PrivateKeyBase::from_data(carol_seed()) }
pub fn carol_public_keys() -> PublicKeyBase { carol_private_keys().public_keys() }

// pub fn example_ledger_identifier() -> ARID { ARID::from_data(hex!("0eda5ce79a2b5619e387f490861a2e7211559029b3b369cf98fb749bd3ba9a5d")) }
// pub fn example_ledger_seed() -> Vec<u8> { hex!("d6737ab34e4e8bb05b6ac035f9fba81a").into() }
// pub fn example_ledger_private_keys() -> PrivateKeyBase { PrivateKeyBase::from_data(&example_ledger_seed()) }
// pub fn example_ledger_public_keys() -> PublicKeyBase { example_ledger_private_keys().public_keys() }

// pub fn state_identifier() -> ARID { ARID::from_data(hex!("04363d5ff99733bc0f1577baba440af1cf344ad9e454fad9d128c00fef6505e8")) }
// pub fn state_seed() -> Vec<u8> { hex!("3e9271f46cdb85a3b584e7220b976918").into() }
// pub fn state_private_keys() -> PrivateKeyBase { PrivateKeyBase::from_data(&state_seed()) }
// pub fn state_public_keys() -> PublicKeyBase { state_private_keys().public_keys() }

pub fn fake_content_key() -> SymmetricKey { SymmetricKey::from_data(hex!("526afd95b2229c5381baec4a1788507a3c4a566ca5cce64543b46ad12aff0035")) }
pub fn fake_nonce() -> Nonce { Nonce::from_data(hex!("4d785658f36c22fb5aed3ac0"))}
