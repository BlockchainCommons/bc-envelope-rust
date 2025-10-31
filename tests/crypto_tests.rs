#![cfg(feature = "encrypt")]

use bc_components::SymmetricKey;
use bc_envelope::prelude::*;
use indoc::indoc;

mod common;
use crate::common::{check_encoding::*, test_data::*};

#[test]
fn plaintext() {
    bc_components::register_tags();

    // Alice sends a plaintext message to Bob.
    let envelope = hello_envelope();
    let ur = envelope.ur();

    #[rustfmt::skip]
    let expected_format = (indoc! {r#"
        "Hello."
    "#}).trim();
    assert_actual_expected!(envelope.format(), expected_format);

    // Alice ➡️ ☁️ ➡️ Bob

    // Bob receives the envelope and reads the message.
    let received_plaintext = Envelope::from_ur(&ur)
        .unwrap()
        .check_encoding()
        .unwrap()
        .extract_subject::<String>()
        .unwrap();
    assert_eq!(received_plaintext, PLAINTEXT_HELLO);
}

#[test]
fn symmetric_encryption() {
    bc_components::register_tags();

    // Alice and Bob have agreed to use this key.
    let key = SymmetricKey::new();

    // Alice sends a message encrypted with the key to Bob.
    let envelope = hello_envelope()
        .encrypt_subject(&key)
        .unwrap()
        .check_encoding()
        .unwrap();
    let ur = envelope.ur();

    #[rustfmt::skip]
    let expected_format = (indoc! {r#"
        ENCRYPTED
    "#}).trim();
    assert_actual_expected!(envelope.format(), expected_format);

    // Alice ➡️ ☁️ ➡️ Bob

    // Bob receives the envelope.
    let received_envelope =
        Envelope::from_ur(&ur).unwrap().check_encoding().unwrap();

    // Bob decrypts and reads the message.
    let received_plaintext = received_envelope
        .decrypt_subject(&key)
        .unwrap()
        .extract_subject::<String>()
        .unwrap();
    assert_eq!(received_plaintext, PLAINTEXT_HELLO);

    // Can't read with no key.
    assert!(received_envelope.extract_subject::<String>().is_err());

    // Can't read with incorrect key.
    assert!(
        received_envelope
            .decrypt_subject(&SymmetricKey::new())
            .is_err()
    );
}

fn round_trip_test(envelope: Envelope) {
    let key = SymmetricKey::new();
    let plaintext_subject = envelope.check_encoding().unwrap();
    let encrypted_subject = plaintext_subject.encrypt_subject(&key).unwrap();
    assert!(encrypted_subject.is_equivalent_to(&plaintext_subject));
    let plaintext_subject2 = encrypted_subject
        .decrypt_subject(&key)
        .unwrap()
        .check_encoding()
        .unwrap();
    assert!(encrypted_subject.is_equivalent_to(&plaintext_subject2));
    assert!(plaintext_subject.is_identical_to(&plaintext_subject2));
}

#[test]
fn encrypt_decrypt() {
    // leaf
    let e = Envelope::new(PLAINTEXT_HELLO);
    round_trip_test(e);

    // node
    let e = Envelope::new("Alice").add_assertion("knows", "Bob");
    round_trip_test(e);

    // wrapped
    let e = Envelope::new("Alice").wrap();
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

#[cfg(all(feature = "signature", feature = "secp256k1"))]
#[test]
fn sign_then_encrypt() {
    bc_components::register_tags();

    // Alice and Bob have agreed to use this key.
    let key = SymmetricKey::new();

    // Alice signs a plaintext message, then encrypts it.
    let envelope = hello_envelope()
        .add_signature(&alice_private_key())
        .check_encoding()
        .unwrap()
        .wrap()
        .check_encoding()
        .unwrap()
        .encrypt_subject(&key)
        .unwrap()
        .check_encoding()
        .unwrap();
    let ur = envelope.ur();

    #[rustfmt::skip]
    let expected_format = (indoc! {r#"
        ENCRYPTED
    "#}).trim();
    assert_actual_expected!(envelope.format(), expected_format);

    // Alice ➡️ ☁️ ➡️ Bob

    // Bob receives the envelope, decrypts it using the shared key, and then
    // validates Alice's signature.
    let received_plaintext = Envelope::from_ur(&ur)
        .unwrap()
        .check_encoding()
        .unwrap()
        .decrypt_subject(&key)
        .unwrap()
        .check_encoding()
        .unwrap()
        .try_unwrap()
        .unwrap()
        .check_encoding()
        .unwrap()
        .verify_signature_from(&alice_public_key());

    // Bob reads the message.
    let received_plaintext = received_plaintext
        .unwrap()
        .extract_subject::<String>()
        .unwrap();
    assert_eq!(received_plaintext, PLAINTEXT_HELLO);
}

#[cfg(all(feature = "signature", feature = "secp256k1"))]
#[test]
fn test_encrypt_then_sign() {
    bc_components::register_tags();

    // Alice and Bob have agreed to use this key.
    let key = SymmetricKey::new();

    // Alice encrypts a plaintext message, then signs it.
    //
    // It doesn't actually matter whether the `encrypt` or `sign` method comes
    // first, as the `encrypt` method transforms the `subject` into its
    // `.encrypted` form, which carries a `Digest` of the plaintext
    // `subject`, while the `sign` method only adds an `Assertion` with the
    // signature of the hash as the `object` of the `Assertion`.
    //
    // Similarly, the `decrypt` method used below can come before or after the
    // `verifySignature` method, as `verifySignature` checks the signature
    // against the `subject`'s hash, which is explicitly present when the
    // subject is in `.encrypted` form and can be calculated when the
    // subject is in `.plaintext` form. The `decrypt` method transforms the
    // subject from its `.encrypted` case to its `.plaintext` case, and also
    // checks that the decrypted plaintext has the same hash as the one
    // associated with the `.encrypted` subject.
    //
    // The end result is the same: the `subject` is encrypted and the signature
    // can be checked before or after decryption.
    //
    // The main difference between this order of operations and the
    // sign-then-encrypt order of operations is that with sign-then-encrypt,
    // the decryption *must* be performed first before the presence of
    // signatures can be known or checked. With this order of operations,
    // the presence of signatures is known before decryption, and may be
    // checked before or after decryption.
    let envelope = hello_envelope()
        .encrypt_subject(&key)
        .unwrap()
        .add_signature(&alice_private_key())
        .check_encoding()
        .unwrap();
    let ur = envelope.ur();

    #[rustfmt::skip]
    let expected_format = (indoc! {r#"
        ENCRYPTED [
            'signed': Signature
        ]
    "#}).trim();
    assert_actual_expected!(envelope.format(), expected_format);

    // Alice ➡️ ☁️ ➡️ Bob

    // Bob receives the envelope, validates Alice's signature, then decrypts the
    // message.
    let received_plaintext = Envelope::from_ur(&ur)
        .unwrap()
        .check_encoding()
        .unwrap()
        .verify_signature_from(&alice_public_key())
        .unwrap()
        .decrypt_subject(&key)
        .unwrap()
        .check_encoding()
        .unwrap()
        .extract_subject::<String>()
        .unwrap();
    // Bob reads the message.
    assert_eq!(received_plaintext, PLAINTEXT_HELLO);
}

#[cfg(all(feature = "recipient", feature = "secp256k1"))]
#[test]
fn test_multi_recipient() {
    // Alice encrypts a message so that it can only be decrypted by Bob or
    // Carol.
    let content_key = SymmetricKey::new();
    let envelope = hello_envelope()
        .encrypt_subject(&content_key)
        .unwrap()
        .add_recipient(&bob_public_key(), &content_key)
        .add_recipient(&carol_public_key(), &content_key)
        .check_encoding()
        .unwrap();
    let ur = envelope.ur();

    #[rustfmt::skip]
    let expected_format = (indoc! {r#"
        ENCRYPTED [
            'hasRecipient': SealedMessage
            'hasRecipient': SealedMessage
        ]
    "#}).trim();
    assert_actual_expected!(envelope.format(), expected_format);

    // Alice ➡️ ☁️ ➡️ Bob
    // Alice ➡️ ☁️ ➡️ Carol

    // The envelope is received
    let received_envelope = Envelope::from_ur(&ur).unwrap();

    // Bob decrypts and reads the message
    let bob_received_plaintext = received_envelope
        .decrypt_subject_to_recipient(&bob_private_key())
        .unwrap()
        .check_encoding()
        .unwrap()
        .extract_subject::<String>()
        .unwrap();
    assert_eq!(bob_received_plaintext, PLAINTEXT_HELLO);

    // Carol decrypts and reads the message
    let carol_received_plaintext = received_envelope
        .decrypt_subject_to_recipient(&carol_private_key())
        .unwrap()
        .check_encoding()
        .unwrap()
        .extract_subject::<String>()
        .unwrap();
    assert_eq!(carol_received_plaintext, PLAINTEXT_HELLO);

    // Alice didn't encrypt it to herself, so she can't read it.
    assert!(
        received_envelope
            .decrypt_subject_to_recipient(&alice_private_key())
            .is_err()
    );
}

#[cfg(all(feature = "signature", feature = "recipient"))]
#[test]
fn test_visible_signature_multi_recipient() {
    // Alice signs a message, and then encrypts it so that it can only be
    // decrypted by Bob or Carol.
    let content_key = SymmetricKey::new();
    let envelope = hello_envelope()
        .add_signature(&alice_private_key())
        .encrypt_subject(&content_key)
        .unwrap()
        .add_recipient(&bob_public_key(), &content_key)
        .add_recipient(&carol_public_key(), &content_key)
        .check_encoding()
        .unwrap();
    let ur = envelope.ur();

    #[rustfmt::skip]
    let expected_format = (indoc! {r#"
        ENCRYPTED [
            'hasRecipient': SealedMessage
            'hasRecipient': SealedMessage
            'signed': Signature
        ]
    "#}).trim();
    assert_actual_expected!(envelope.format(), expected_format);

    // Alice ➡️ ☁️ ➡️ Bob
    // Alice ➡️ ☁️ ➡️ Carol

    // The envelope is received
    let received_envelope = Envelope::from_ur(&ur).unwrap();

    // Bob validates Alice's signature, then decrypts and reads the message
    let bob_received_plaintext = received_envelope
        .verify_signature_from(&alice_public_key())
        .unwrap()
        .decrypt_subject_to_recipient(&bob_private_key())
        .unwrap()
        .check_encoding()
        .unwrap()
        .extract_subject::<String>()
        .unwrap();
    assert_eq!(bob_received_plaintext, PLAINTEXT_HELLO);

    // Carol validates Alice's signature, then decrypts and reads the message
    let carol_received_plaintext = received_envelope
        .verify_signature_from(&alice_public_key())
        .unwrap()
        .decrypt_subject_to_recipient(&carol_private_key())
        .unwrap()
        .check_encoding()
        .unwrap()
        .extract_subject::<String>()
        .unwrap();
    assert_eq!(carol_received_plaintext, PLAINTEXT_HELLO);

    // Alice didn't encrypt it to herself, so she can't read it.
    assert!(
        received_envelope
            .decrypt_subject_to_recipient(&alice_private_key())
            .is_err()
    );
}

#[cfg(all(feature = "signature", feature = "recipient"))]
#[test]
fn test_hidden_signature_multi_recipient() {
    // Alice signs a message, and then encloses it in another envelope before
    // encrypting it so that it can only be decrypted by Bob or Carol. This
    // hides Alice's signature, and requires recipients to decrypt the
    // subject before they are able to validate the signature.
    let content_key = SymmetricKey::new();
    let envelope = hello_envelope()
        .add_signature(&alice_private_key())
        .wrap()
        .encrypt_subject(&content_key)
        .unwrap()
        .add_recipient(&bob_public_key(), &content_key)
        .add_recipient(&carol_public_key(), &content_key)
        .check_encoding()
        .unwrap();
    let ur = envelope.ur();

    #[rustfmt::skip]
    let expected_format = (indoc! {r#"
        ENCRYPTED [
            'hasRecipient': SealedMessage
            'hasRecipient': SealedMessage
        ]
    "#}).trim();
    assert_actual_expected!(envelope.format(), expected_format);

    // Alice ➡️ ☁️ ➡️ Bob
    // Alice ➡️ ☁️ ➡️ Carol

    // The envelope is received
    let received_envelope = Envelope::from_ur(&ur).unwrap();

    // Bob decrypts the envelope, then extracts the inner envelope and validates
    // Alice's signature, then reads the message
    let bob_received_plaintext = received_envelope
        .decrypt_subject_to_recipient(&bob_private_key())
        .unwrap()
        .try_unwrap()
        .unwrap()
        .check_encoding()
        .unwrap()
        .verify_signature_from(&alice_public_key())
        .unwrap()
        .extract_subject::<String>()
        .unwrap();
    assert_eq!(bob_received_plaintext, PLAINTEXT_HELLO);

    // Carol decrypts the envelope, then extracts the inner envelope and
    // validates Alice's signature, then reads the message
    let carol_received_plaintext = received_envelope
        .decrypt_subject_to_recipient(&carol_private_key())
        .unwrap()
        .try_unwrap()
        .unwrap()
        .check_encoding()
        .unwrap()
        .verify_signature_from(&alice_public_key())
        .unwrap()
        .extract_subject::<String>()
        .unwrap();
    assert_eq!(carol_received_plaintext, PLAINTEXT_HELLO);

    // Alice didn't encrypt it to herself, so she can't read it.
    assert!(
        received_envelope
            .decrypt_subject_to_recipient(&alice_private_key())
            .is_err()
    );
}
#[cfg(feature = "secret")]
#[test]
fn test_secret_1() {
    use bc_components::KeyDerivationMethod;

    bc_envelope::register_tags();
    let bob_password = "correct horse battery staple";
    // Alice encrypts a message so that it can only be decrypted by Bob's
    // password.
    let envelope = hello_envelope()
        .lock(KeyDerivationMethod::HKDF, bob_password)
        .unwrap();
    envelope.check_encoding().unwrap();
    let ur = envelope.ur();
    #[rustfmt::skip]
    let expected_format = (indoc! {r#"
        ENCRYPTED [
            'hasSecret': EncryptedKey(HKDF(SHA256))
        ]
    "#}).trim();
    assert_actual_expected!(envelope.format(), expected_format);
    // Alice ➡️ ☁️ ➡️ Bob, Eve
    // The envelope is received
    let received_envelope = Envelope::from_ur(&ur).unwrap();
    // Bob decrypts and reads the message
    let bob_received_plaintext = received_envelope
        .unlock(bob_password)
        .unwrap()
        .check_encoding()
        .unwrap()
        .extract_subject::<String>()
        .unwrap();
    assert_eq!(bob_received_plaintext, PLAINTEXT_HELLO);

    // Eve tries to decrypt the message with a different password
    assert!(received_envelope.unlock("wrong password").is_err());
}

#[cfg(feature = "secret")]
#[test]
fn test_secret_2() {
    use bc_components::KeyDerivationMethod;

    bc_envelope::register_tags();

    // Alice encrypts a message so that it can be decrypted by three specific
    // passwords.
    let bob_password = "correct horse battery staple";
    let carol_password = "Able was I ere I saw Elba";
    let gracy_password = "Madam, in Eden, I'm Adam";
    let content_key = SymmetricKey::new();
    let envelope = hello_envelope()
        .encrypt_subject(&content_key)
        .unwrap()
        .add_secret(KeyDerivationMethod::HKDF, bob_password, &content_key)
        .unwrap()
        .add_secret(KeyDerivationMethod::Scrypt, carol_password, &content_key)
        .unwrap()
        .add_secret(KeyDerivationMethod::Argon2id, gracy_password, &content_key)
        .unwrap()
        .check_encoding()
        .unwrap();
    let ur = envelope.ur();
    #[rustfmt::skip]
    let expected_format = (indoc! {r#"
        ENCRYPTED [
            'hasSecret': EncryptedKey(Argon2id)
            'hasSecret': EncryptedKey(HKDF(SHA256))
            'hasSecret': EncryptedKey(Scrypt)
        ]
    "#}).trim();
    assert_actual_expected!(envelope.format(), expected_format);
    // Alice ➡️ ☁️ ➡️ Bob, Carol, Eve
    // The envelope is received
    let received_envelope = Envelope::from_ur(&ur).unwrap();
    // Bob decrypts and reads the message
    let bob_received_plaintext = received_envelope
        .unlock_subject(bob_password)
        .unwrap()
        .check_encoding()
        .unwrap()
        .extract_subject::<String>()
        .unwrap();
    assert_eq!(bob_received_plaintext, PLAINTEXT_HELLO);
    // Carol decrypts and reads the message
    let carol_received_plaintext = received_envelope
        .unlock_subject(carol_password)
        .unwrap()
        .check_encoding()
        .unwrap()
        .extract_subject::<String>()
        .unwrap();
    assert_eq!(carol_received_plaintext, PLAINTEXT_HELLO);
    let gracy_received_plaintext = received_envelope
        .unlock_subject(gracy_password)
        .unwrap()
        .check_encoding()
        .unwrap()
        .extract_subject::<String>()
        .unwrap();
    assert_eq!(gracy_received_plaintext, PLAINTEXT_HELLO);
    // Eve tries to decrypt the message with a different password
    assert!(received_envelope.unlock_subject("wrong password").is_err());
}
