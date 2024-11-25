#![cfg(feature = "signature")]

use indoc::indoc;
use bc_envelope::prelude::*;

mod common;
use crate::common::test_data::*;
use crate::common::check_encoding::*;

#[test]
fn test_signed_plaintext() {
    // Alice sends a signed plaintext message to Bob.
    let envelope = hello_envelope()
        .add_signature(&alice_private_key())
        .check_encoding().unwrap();
    let ur = envelope.ur();

    let expected_format = indoc! {r#"
    "Hello." [
        'signed': Signature
    ]
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);

    // Alice ➡️ ☁️ ➡️ Bob

    // Bob receives the envelope.
    let received_envelope = Envelope::from_ur(&ur).unwrap()
        .check_encoding().unwrap();

    // Bob receives the message, validates Alice's signature, and reads the message.
    let received_plaintext = received_envelope
        .verify_signature_from(&alice_public_key());
    let received_plaintext = received_plaintext.unwrap()
        .extract_subject::<String>().unwrap();
    assert_eq!(received_plaintext, "Hello.");

    // Confirm that it wasn't signed by Carol.
    assert!(received_envelope.verify_signature_from(&carol_public_key()).is_err());

    // Confirm that it was signed by Alice OR Carol.
    received_envelope.verify_signatures_from_threshold(&[&alice_public_key(), &carol_public_key()], Some(1)).unwrap();

    // Confirm that it was not signed by Alice AND Carol.
    assert!(received_envelope.verify_signatures_from_threshold(&[&alice_public_key(), &carol_public_key()], Some(2)).is_err());
}

#[test]
fn multisigned_plaintext() {
    bc_components::register_tags();

    // Alice and Carol jointly send a signed plaintext message to Bob.
    let envelope = hello_envelope()
        .add_signatures(&[&alice_private_key(), &carol_private_key()])
        .check_encoding().unwrap();
    let ur = envelope.ur();

    let expected_format = indoc! {r#"
    "Hello." [
        'signed': Signature
        'signed': Signature
    ]
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);

    // Alice & Carol ➡️ ☁️ ➡️ Bob

    // Bob receives the envelope and verifies the message was signed by both Alice and Carol.
    let received_plaintext = Envelope::from_ur(&ur).unwrap()
        .check_encoding().unwrap()
        .verify_signatures_from(&[&alice_public_key(), &carol_public_key()]);

    // Bob reads the message.
    let received_plaintext = received_plaintext.unwrap()
        .extract_subject::<String>().unwrap();
    assert_eq!(received_plaintext, PLAINTEXT_HELLO);
}
