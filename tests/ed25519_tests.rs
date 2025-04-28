#![cfg(feature = "signature")]

use bc_envelope::prelude::*;
use indoc::indoc;

mod common;
use crate::common::test_data::*;
use crate::common::check_encoding::*;

#[test]
fn test_ed25519_signed_plaintext() {
    bc_components::register_tags();

    let alice_private_key = alice_private_key().ed25519_signing_private_key();
    let alice_public_key = alice_private_key.public_key().unwrap();

    // Alice sends a signed plaintext message to Bob.
    let envelope = hello_envelope().add_signature(&alice_private_key).check_encoding().unwrap();
    let ur = envelope.ur();

    #[rustfmt::skip]
    let expected_format = indoc! {r#"
        "Hello." [
            'signed': Signature(Ed25519)
        ]
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);

    // Alice ➡️ ☁️ ➡️ Bob

    // Bob receives the envelope.
    let received_envelope = Envelope::from_ur(&ur).unwrap().check_encoding().unwrap();

    // Bob receives the message, validates Alice's signature, and reads the message.
    let received_plaintext = received_envelope.verify_signature_from(&alice_public_key);
    let received_plaintext = received_plaintext.unwrap().extract_subject::<String>().unwrap();
    assert_eq!(received_plaintext, "Hello.");

    // Confirm that it wasn't signed by Carol.
    let carol_public_key = carol_private_key().ed25519_signing_private_key().public_key().unwrap();
    assert!(received_envelope.verify_signature_from(&carol_public_key).is_err());

    // Confirm that it was signed by Alice OR Carol.
    received_envelope
        .verify_signatures_from_threshold(&[&alice_public_key, &carol_public_key], Some(1))
        .unwrap();

    // Confirm that it was not signed by Alice AND Carol.
    assert!(
        received_envelope
            .verify_signatures_from_threshold(&[&alice_public_key, &carol_public_key], Some(2))
            .is_err()
    );
}
