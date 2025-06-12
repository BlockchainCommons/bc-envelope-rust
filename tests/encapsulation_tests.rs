#![cfg(feature = "recipient")]

use bc_components::EncapsulationScheme;
use bc_envelope::prelude::*;

mod common;
use crate::common::{check_encoding::*, test_data::*};

fn test_scheme(scheme: EncapsulationScheme) {
    let (private_key, public_key) = scheme.keypair();
    let envelope = hello_envelope();
    let encrypted_envelope = envelope
        .encrypt_to_recipient(&public_key)
        .check_encoding()
        .unwrap();
    // println!("{}", encrypted_envelope.format());
    let decrypted_envelope = encrypted_envelope
        .decrypt_to_recipient(&private_key)
        .unwrap();
    assert_eq!(
        envelope.structural_digest(),
        decrypted_envelope.structural_digest()
    );
}

#[test]
fn test_encapsulation() {
    bc_components::register_tags();

    test_scheme(EncapsulationScheme::X25519);
    test_scheme(EncapsulationScheme::MLKEM512);
    test_scheme(EncapsulationScheme::MLKEM768);
    test_scheme(EncapsulationScheme::MLKEM1024);
}
