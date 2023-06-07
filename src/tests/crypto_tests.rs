use std::rc::Rc;

use bc_components::SymmetricKey;
use bc_ur::{UREncodable, URDecodable};
use indoc::indoc;

use crate::Envelope;

use super::*;

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

#[test]
fn test_signed_plaintext() {
    // Alice sends a signed plaintext message to Bob.
    let envelope = hello_envelope()
        .sign_with(&alice_private_keys())
        .check_encoding().unwrap();
    let ur = envelope.ur();

    let expected_format = indoc! {r#"
    "Hello." [
        verifiedBy: Signature
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

#[test]
fn multisigned_plaintext() {
    // Alice and Carol jointly send a signed plaintext message to Bob.
    let envelope = hello_envelope()
        .sign_with_keys(&[&alice_private_keys(), &carol_private_keys()])
        .check_encoding().unwrap();
    let ur = envelope.ur();

    let expected_format = indoc! {r#"
    "Hello." [
        verifiedBy: Signature
        verifiedBy: Signature
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

#[test]
fn encrypt_decrypt() {
    let key = SymmetricKey::new();
    let plaintext_envelope = hello_envelope()
        .check_encoding().unwrap();
    let encrypted_envelope = plaintext_envelope.clone()
        .encrypt_subject(&key).unwrap()
        .check_encoding().unwrap();
    assert!(plaintext_envelope.is_equivalent_to(encrypted_envelope.clone()));
    let plaintext_envelope2 = encrypted_envelope.clone()
        .decrypt_subject(&key).unwrap()
        .check_encoding().unwrap();
    assert!(encrypted_envelope.is_equivalent_to(plaintext_envelope2));
}

#[test]
fn sign_then_encrypt() {
    // Alice and Bob have agreed to use this key.
    let key = SymmetricKey::new();

    // Alice signs a plaintext message, then encrypts it.
    let envelope = hello_envelope()
        .sign_with(&alice_private_keys())
        .check_encoding().unwrap()
        .wrap_envelope().check_encoding().unwrap()
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
        .unwrap_envelope().unwrap().check_encoding().unwrap()
        .verify_signature_from(&alice_public_keys());

    // Bob reads the message.
    let received_plaintext = received_plaintext.unwrap()
        .extract_subject::<String>().unwrap();
    assert_eq!(*received_plaintext, PLAINTEXT_HELLO);
}

/*```swift
    func testEncryptThenSign() throws {
        // Alice and Bob have agreed to use this key.
        let key = SymmetricKey()

        let envelope = try Envelope(plaintextHello)
            .encryptSubject(with: key).checkEncoding()
            .sign(with: alicePrivateKeys).checkEncoding()
        let ur = envelope.ur

        let expectedFormat =
        """
        ENCRYPTED [
            verifiedBy: Signature
        ]
        """
        XCTAssertEqual(envelope.format(), expectedFormat)

//        print(envelope.taggedCBOR.diagnostic())
//        print(envelope.taggedCBOR.hex())
//        print(envelope.ur)

        // Alice ➡️ ☁️ ➡️ Bob

        // Bob receives the envelope, validates Alice's signature, then decrypts the message.
        let receivedPlaintext = try Envelope(ur: ur).checkEncoding()
            .verifySignature(from: alicePublicKeys)
            .decryptSubject(with: key).checkEncoding()
            .extractSubject(String.self)
        // Bob reads the message.
        XCTAssertEqual(receivedPlaintext, plaintextHello)
    }
``` */

#[test]
fn test_encrypt_then_sign() {
    // Alice and Bob have agreed to use this key.
    let key = SymmetricKey::new();

    // Alice encryptes a plaintext message, then signs it.
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
        verifiedBy: Signature
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
