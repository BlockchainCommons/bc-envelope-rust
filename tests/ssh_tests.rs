#![cfg(feature = "ssh")]
use bc_components::SigningOptions;
use bc_envelope::prelude::*;
use indoc::indoc;
use ssh_key::{Algorithm as SSHAlgorithm, HashAlg};

mod common;
use crate::common::{check_encoding::*, test_data::*};

#[test]
fn test_ssh_signed_plaintext() {
    bc_components::register_tags();

    let alice_ssh_private_key = alice_private_key()
        .ssh_signing_private_key(SSHAlgorithm::Ed25519, "alice@example.com")
        .unwrap();
    let alice_ssh_public_key = alice_ssh_private_key.public_key().unwrap();

    // Alice sends a signed plaintext message to Bob.
    let options = SigningOptions::Ssh {
        namespace: "test".to_string(),
        hash_alg: HashAlg::Sha256,
    };
    let envelope = hello_envelope()
        .add_signature_opt(&alice_ssh_private_key, Some(options), None)
        .check_encoding()
        .unwrap();
    let ur = envelope.ur();

    #[rustfmt::skip]
    let expected_format = indoc! {r#"
        "Hello." [
            'signed': Signature(SshEd25519)
        ]
    "#}.trim();
    assert_actual_expected!(envelope.format(), expected_format);

    // Alice ➡️ ☁️ ➡️ Bob

    // Bob receives the envelope.
    let received_envelope =
        Envelope::from_ur(&ur).unwrap().check_encoding().unwrap();

    // Bob receives the message, validates Alice's signature, and reads the
    // message.
    let received_plaintext =
        received_envelope.verify_signature_from(&alice_ssh_public_key);
    let received_plaintext = received_plaintext
        .unwrap()
        .extract_subject::<String>()
        .unwrap();
    assert_eq!(received_plaintext, "Hello.");

    // Confirm that it wasn't signed by Carol.
    assert!(
        received_envelope
            .verify_signature_from(&carol_public_key())
            .is_err()
    );

    // Confirm that it was signed by Alice OR Carol.
    received_envelope
        .verify_signatures_from_threshold(
            &[&alice_ssh_public_key, &carol_public_key()],
            Some(1),
        )
        .unwrap();

    // Confirm that it was not signed by Alice AND Carol.
    assert!(
        received_envelope
            .verify_signatures_from_threshold(
                &[&alice_ssh_public_key, &carol_public_key()],
                Some(2)
            )
            .is_err()
    );
}
