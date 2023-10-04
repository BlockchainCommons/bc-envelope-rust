use std::rc::Rc;

use bc_components::SymmetricKey;
use bc_ur::prelude::*;
use indoc::indoc;

use bc_envelope::prelude::*;

mod common;
use crate::common::test_data::*;
use crate::common::check_encoding::*;

#[test]
fn plaintext() {
    // Alice sends a plaintext message to Bob.
    let envelope = hello_envelope();
    let ur = envelope.ur();

    let expected_format = indoc! {r#"
    "Hello."
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);

    // Alice ➡️ ☁️ ➡️ Bob

    // Bob receives the envelope and reads the message.
    let received_plaintext = Rc::new(Envelope::from_ur(&ur).unwrap())
        .check_encoding().unwrap()
        .extract_subject::<String>().unwrap();
    assert_eq!(*received_plaintext, PLAINTEXT_HELLO);
}

#[cfg(feature = "signature")]
#[test]
fn test_signed_plaintext() {
    // Alice sends a signed plaintext message to Bob.
    let envelope = hello_envelope()
        .sign_with(&alice_private_keys())
        .check_encoding().unwrap();
    let ur = envelope.ur();

    let expected_format = indoc! {r#"
    "Hello." [
        'verifiedBy': Signature
    ]
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);

    // Alice ➡️ ☁️ ➡️ Bob

    // Bob receives the envelope.
    let received_envelope = Rc::new(Envelope::from_ur(&ur).unwrap())
        .check_encoding().unwrap();

    // Bob receives the message, validates Alice's signature, and reads the message.
    let received_plaintext = received_envelope.clone()
        .verify_signature_from(&alice_public_keys());
    let received_plaintext = received_plaintext.unwrap()
        .extract_subject::<String>().unwrap();
    assert_eq!(*received_plaintext, "Hello.");

    // Confirm that it wasn't signed by Carol.
    assert!(received_envelope.clone().verify_signature_from(&carol_public_keys()).is_err());

    // Confirm that it was signed by Alice OR Carol.
    received_envelope.clone().verify_signatures_from_threshold(&[&alice_public_keys(), &carol_public_keys()], Some(1)).unwrap();

    // Confirm that it was not signed by Alice AND Carol.
    assert!(received_envelope.verify_signatures_from_threshold(&[&alice_public_keys(), &carol_public_keys()], Some(2)).is_err());
}

#[cfg(feature = "signature")]
#[test]
fn multisigned_plaintext() {
    // Alice and Carol jointly send a signed plaintext message to Bob.
    let envelope = hello_envelope()
        .sign_with_keys(&[&alice_private_keys(), &carol_private_keys()])
        .check_encoding().unwrap();
    let ur = envelope.ur();

    let expected_format = indoc! {r#"
    "Hello." [
        'verifiedBy': Signature
        'verifiedBy': Signature
    ]
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);

    // Alice & Carol ➡️ ☁️ ➡️ Bob

    // Bob receives the envelope and verifies the message was signed by both Alice and Carol.
    let received_plaintext = Rc::new(Envelope::from_ur(&ur).unwrap())
        .check_encoding().unwrap()
        .verify_signatures_from(&[&alice_public_keys(), &carol_public_keys()]);

    // Bob reads the message.
    let received_plaintext = received_plaintext.unwrap()
        .extract_subject::<String>().unwrap();
    assert_eq!(*received_plaintext, PLAINTEXT_HELLO);
}

#[test]
fn symmetric_encryption() {
    // Alice and Bob have agreed to use this key.
    let key = SymmetricKey::new();

    // Alice sends a message encrypted with the key to Bob.
    let envelope = hello_envelope()
        .encrypt_subject(&key).unwrap()
        .check_encoding().unwrap();
    let ur = envelope.ur();

    let expected_format = indoc! {r#"
    ENCRYPTED
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);

    // Alice ➡️ ☁️ ➡️ Bob

    // Bob receives the envelope.
    let received_envelope = Rc::new(Envelope::from_ur(&ur).unwrap())
        .check_encoding().unwrap();

    // Bob decrypts and reads the message.
    let received_plaintext = received_envelope.clone()
        .decrypt_subject(&key).unwrap()
        .extract_subject::<String>().unwrap();
    assert_eq!(*received_plaintext, PLAINTEXT_HELLO);

    // Can't read with no key.
    assert!(received_envelope.extract_subject::<String>().is_err());

    // Can't read with incorrect key.
    assert!(received_envelope.decrypt_subject(&SymmetricKey::new()).is_err());
}

fn round_trip_test(envelope: Rc<Envelope>) {
    let key = SymmetricKey::new();
    let plaintext_subject = envelope.check_encoding().unwrap();
    let encrypted_subject = plaintext_subject.clone()
        .encrypt_subject(&key).unwrap();
    assert!(encrypted_subject.clone().is_equivalent_to(plaintext_subject.clone()));
    let plaintext_subject2 = encrypted_subject.clone()
        .decrypt_subject(&key).unwrap()
        .check_encoding().unwrap();
    assert!(encrypted_subject.is_equivalent_to(plaintext_subject2.clone()));
    assert!(plaintext_subject.is_identical_to(plaintext_subject2));
}

#[test]
fn encrypt_decrypt() {
    // leaf
    let e = Envelope::new(PLAINTEXT_HELLO);
    round_trip_test(e);

    // node
    let e = Envelope::new("Alice")
        .add_assertion("knows", "Bob");
    round_trip_test(e);

    // wrapped
    let e = Envelope::new("Alice")
        .wrap_envelope();
    round_trip_test(e);

    // known value
    let e = Envelope::new(known_values::IS_A);
    round_trip_test(e);

    // assertion
    let e = Envelope::new_assertion("knows", "Bob");
    round_trip_test(e);

    #[cfg(feature = "compress")]
    {
        // compressed
        let e = Envelope::new(PLAINTEXT_HELLO).compress().unwrap();
        round_trip_test(e);
    }
}

#[cfg(feature = "signature")]
#[test]
fn sign_then_encrypt() {
    // Alice and Bob have agreed to use this key.
    let key = SymmetricKey::new();

    // Alice signs a plaintext message, then encrypts it.
    let envelope = hello_envelope()
        .sign_with(&alice_private_keys())
        .check_encoding().unwrap()
        .wrap_envelope()
        .check_encoding().unwrap()
        .encrypt_subject(&key).unwrap()
        .check_encoding().unwrap();
    let ur = envelope.ur();

    let expected_format = indoc! {r#"
    ENCRYPTED
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);

    // Alice ➡️ ☁️ ➡️ Bob

    // Bob receives the envelope, decrypts it using the shared key, and then validates Alice's signature.
    let received_plaintext = Rc::new(Envelope::from_ur(&ur).unwrap())
        .check_encoding().unwrap()
        .decrypt_subject(&key).unwrap()
        .check_encoding().unwrap()
        .unwrap_envelope().unwrap()
        .check_encoding().unwrap()
        .verify_signature_from(&alice_public_keys());

    // Bob reads the message.
    let received_plaintext = received_plaintext.unwrap()
        .extract_subject::<String>().unwrap();
    assert_eq!(*received_plaintext, PLAINTEXT_HELLO);
}

#[cfg(feature = "signature")]
#[test]
fn test_encrypt_then_sign() {
    // Alice and Bob have agreed to use this key.
    let key = SymmetricKey::new();

    // Alice encrypts a plaintext message, then signs it.
    //
    // It doesn't actually matter whether the `encrypt` or `sign` method comes first,
    // as the `encrypt` method transforms the `subject` into its `.encrypted` form,
    // which carries a `Digest` of the plaintext `subject`, while the `sign` method
    // only adds an `Assertion` with the signature of the hash as the `object` of the
    // `Assertion`.
    //
    // Similarly, the `decrypt` method used below can come before or after the
    // `verifySignature` method, as `verifySignature` checks the signature against
    // the `subject`'s hash, which is explicitly present when the subject is in
    // `.encrypted` form and can be calculated when the subject is in `.plaintext`
    // form. The `decrypt` method transforms the subject from its `.encrypted` case to
    // its `.plaintext` case, and also checks that the decrypted plaintext has the same
    // hash as the one associated with the `.encrypted` subject.
    //
    // The end result is the same: the `subject` is encrypted and the signature can be
    // checked before or after decryption.
    //
    // The main difference between this order of operations and the sign-then-encrypt
    // order of operations is that with sign-then-encrypt, the decryption *must*
    // be performed first before the presence of signatures can be known or checked.
    // With this order of operations, the presence of signatures is known before
    // decryption, and may be checked before or after decryption.
    let envelope = hello_envelope()
        .encrypt_subject(&key).unwrap()
        .sign_with(&alice_private_keys())
        .check_encoding().unwrap();
    let ur = envelope.ur();

    let expected_format = indoc! {r#"
    ENCRYPTED [
        'verifiedBy': Signature
    ]
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);

    // Alice ➡️ ☁️ ➡️ Bob

    // Bob receives the envelope, validates Alice's signature, then decrypts the message.
    let received_plaintext = Rc::new(Envelope::from_ur(&ur).unwrap())
        .check_encoding().unwrap()
        .verify_signature_from(&alice_public_keys()).unwrap()
        .decrypt_subject(&key).unwrap()
        .check_encoding().unwrap()
        .extract_subject::<String>().unwrap();
    // Bob reads the message.
    assert_eq!(*received_plaintext, PLAINTEXT_HELLO);
}

#[cfg(feature = "recipient")]
#[test]
fn test_multi_recipient() {
    // Alice encrypts a message so that it can only be decrypted by Bob or Carol.
    let content_key = SymmetricKey::new();
    let envelope = hello_envelope()
        .encrypt_subject(&content_key).unwrap()
        .add_recipient(&bob_public_keys(), &content_key)
        .add_recipient(&carol_public_keys(), &content_key)
        .check_encoding().unwrap();
    let ur = envelope.ur();

    let expected_format = indoc! {r#"
    ENCRYPTED [
        'hasRecipient': SealedMessage
        'hasRecipient': SealedMessage
    ]
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);

    // Alice ➡️ ☁️ ➡️ Bob
    // Alice ➡️ ☁️ ➡️ Carol

    // The envelope is received
    let received_envelope = Rc::new(Envelope::from_ur(&ur).unwrap());

    // Bob decrypts and reads the message
    let bob_received_plaintext = received_envelope.clone()
        .decrypt_to_recipient(&bob_private_keys()).unwrap()
        .check_encoding().unwrap()
        .extract_subject::<String>().unwrap();
    assert_eq!(*bob_received_plaintext, PLAINTEXT_HELLO);

    // Carol decrypts and reads the message
    let carol_received_plaintext = received_envelope.clone()
        .decrypt_to_recipient(&carol_private_keys()).unwrap()
        .check_encoding().unwrap()
        .extract_subject::<String>().unwrap();
    assert_eq!(*carol_received_plaintext, PLAINTEXT_HELLO);

    // Alice didn't encrypt it to herself, so she can't read it.
    assert!(received_envelope.decrypt_to_recipient(&alice_private_keys()).is_err());
}

#[cfg(all(feature = "signature", feature = "recipient"))]
#[test]
fn test_visible_signature_multi_recipient() {
    // Alice signs a message, and then encrypts it so that it can only be decrypted by Bob or Carol.
    let content_key = SymmetricKey::new();
    let envelope = hello_envelope()
        .sign_with(&alice_private_keys())
        .encrypt_subject(&content_key).unwrap()
        .add_recipient(&bob_public_keys(), &content_key)
        .add_recipient(&carol_public_keys(), &content_key)
        .check_encoding().unwrap();
    let ur = envelope.ur();

    let expected_format = indoc! {r#"
    ENCRYPTED [
        'hasRecipient': SealedMessage
        'hasRecipient': SealedMessage
        'verifiedBy': Signature
    ]
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);

    // Alice ➡️ ☁️ ➡️ Bob
    // Alice ➡️ ☁️ ➡️ Carol

    // The envelope is received
    let received_envelope = Rc::new(Envelope::from_ur(&ur).unwrap());

    // Bob validates Alice's signature, then decrypts and reads the message
    let bob_received_plaintext = received_envelope.clone()
        .verify_signature_from(&alice_public_keys()).unwrap()
        .decrypt_to_recipient(&bob_private_keys()).unwrap()
        .check_encoding().unwrap()
        .extract_subject::<String>().unwrap();
    assert_eq!(*bob_received_plaintext, PLAINTEXT_HELLO);

    // Carol validates Alice's signature, then decrypts and reads the message
    let carol_received_plaintext = received_envelope.clone()
        .verify_signature_from(&alice_public_keys()).unwrap()
        .decrypt_to_recipient(&carol_private_keys()).unwrap()
        .check_encoding().unwrap()
        .extract_subject::<String>().unwrap();
    assert_eq!(*carol_received_plaintext, PLAINTEXT_HELLO);

    // Alice didn't encrypt it to herself, so she can't read it.
    assert!(received_envelope.decrypt_to_recipient(&alice_private_keys()).is_err());
}

#[cfg(all(feature = "signature", feature = "recipient"))]
#[test]
fn test_hidden_signature_multi_recipient() {
    // Alice signs a message, and then encloses it in another envelope before
    // encrypting it so that it can only be decrypted by Bob or Carol. This hides
    // Alice's signature, and requires recipients to decrypt the subject before they
    // are able to validate the signature.
    let content_key = SymmetricKey::new();
    let envelope = hello_envelope()
        .sign_with(&alice_private_keys())
        .wrap_envelope()
        .encrypt_subject(&content_key).unwrap()
        .add_recipient(&bob_public_keys(), &content_key)
        .add_recipient(&carol_public_keys(), &content_key)
        .check_encoding().unwrap();
    let ur = envelope.ur();

    let expected_format = indoc! {r#"
    ENCRYPTED [
        'hasRecipient': SealedMessage
        'hasRecipient': SealedMessage
    ]
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);

    // Alice ➡️ ☁️ ➡️ Bob
    // Alice ➡️ ☁️ ➡️ Carol

    // The envelope is received
    let received_envelope = Rc::new(Envelope::from_ur(&ur).unwrap());

    // Bob decrypts the envelope, then extracts the inner envelope and validates
    // Alice's signature, then reads the message
    let bob_received_plaintext = received_envelope.clone()
        .decrypt_to_recipient(&bob_private_keys()).unwrap()
        .unwrap_envelope().unwrap()
        .check_encoding().unwrap()
        .verify_signature_from(&alice_public_keys()).unwrap()
        .extract_subject::<String>().unwrap();
    assert_eq!(*bob_received_plaintext, PLAINTEXT_HELLO);

    // Carol decrypts the envelope, then extracts the inner envelope and validates
    // Alice's signature, then reads the message
    let carol_received_plaintext = received_envelope.clone()
        .decrypt_to_recipient(&carol_private_keys()).unwrap()
        .unwrap_envelope().unwrap()
        .check_encoding().unwrap()
        .verify_signature_from(&alice_public_keys()).unwrap()
        .extract_subject::<String>().unwrap();
    assert_eq!(*carol_received_plaintext, PLAINTEXT_HELLO);

    // Alice didn't encrypt it to herself, so she can't read it.
    assert!(received_envelope.decrypt_to_recipient(&alice_private_keys()).is_err());
}
