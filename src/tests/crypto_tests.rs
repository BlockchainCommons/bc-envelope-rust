use std::rc::Rc;

use bc_components::{SymmetricKey, SSKRGroupSpec, SSKRSpec};
use bc_ur::{UREncodable, URDecodable};
use indoc::indoc;
use hex_literal::hex;

use crate::{Envelope, Enclosable};
use super::{test_data::*, Seed};

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
        hasRecipient: SealedMessage
        hasRecipient: SealedMessage
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
        hasRecipient: SealedMessage
        hasRecipient: SealedMessage
        verifiedBy: Signature
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
        hasRecipient: SealedMessage
        hasRecipient: SealedMessage
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

/*```swift
    func testSSKR() throws {
        // Dan has a cryptographic seed he wants to backup using a social recovery scheme.
        // The seed includes metadata he wants to back up also, making it too large to fit
        // into a basic SSKR share.
        var danSeed = Seed(data: ‡"59f2293a5bce7d4de59e71b4207ac5d2")!
        danSeed.name = "Dark Purple Aqua Love"
        danSeed.creationDate = try! Date(iso8601: "2021-02-24")
        danSeed.note = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."

        // Dan encrypts the seed and then splits the content key into a single group
        // 2-of-3. This returns an array of arrays of Envelope, the outer arrays
        // representing SSKR groups and the inner array elements each holding the encrypted
        // seed and a single share.
        let contentKey = SymmetricKey()
        let seedEnvelope = Envelope(danSeed)
        let encryptedSeedEnvelope = try seedEnvelope
            .encryptSubject(with: contentKey)

        let envelopes = encryptedSeedEnvelope
            .split(groupThreshold: 1, groups: [(2, 3)], contentKey: contentKey)

        // Flattening the array of arrays gives just a single array of all the envelopes
        // to be distributed.
        let sentEnvelopes = envelopes.flatMap { $0 }
        let sentURs = sentEnvelopes.map { $0.ur }

        let expectedFormat =
        """
        ENCRYPTED [
            sskrShare: SSKRShare
        ]
        """
        XCTAssertEqual(sentEnvelopes[0].format(), expectedFormat)

        // Dan sends one envelope to each of Alice, Bob, and Carol.

        // Dan ➡️ ☁️ ➡️ Alice
        // Dan ➡️ ☁️ ➡️ Bob
        // Dan ➡️ ☁️ ➡️ Carol

        // let aliceEnvelope = try Envelope(ur: sentURs[0]) // UNRECOVERED
        let bobEnvelope = try Envelope(ur: sentURs[1])
        let carolEnvelope = try Envelope(ur: sentURs[2])

        // At some future point, Dan retrieves two of the three envelopes so he can recover his seed.
        let recoveredEnvelopes = [bobEnvelope, carolEnvelope]
        let a = try Envelope(shares: recoveredEnvelopes)
        let recoveredSeed = try a
            .extractSubject(Seed.self)

        // The recovered seed is correct.
        XCTAssertEqual(danSeed.data, recoveredSeed.data)
        XCTAssertEqual(danSeed.creationDate, recoveredSeed.creationDate)
        XCTAssertEqual(danSeed.name, recoveredSeed.name)
        XCTAssertEqual(danSeed.note, recoveredSeed.note)

        // Attempting to recover with only one of the envelopes won't work.
        XCTAssertThrowsError(try Envelope(shares: [bobEnvelope]))
    }
``` */

#[test]
fn test_sskr() {
    // Dan has a cryptographic seed he wants to backup using a social recovery scheme.
    // The seed includes metadata he wants to back up also, making it too large to fit
    // into a basic SSKR share.
    let mut dan_seed = Seed::new(hex!("59f2293a5bce7d4de59e71b4207ac5d2"));
    dan_seed.set_name("Dark Purple Aqua Love");
    dan_seed.set_creation_date(Some(dcbor::Date::new_from_string("2021-02-24").unwrap()));
    dan_seed.set_note("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.");

    // Dan encrypts the seed and then splits the content key into a single group
    // 2-of-3. This returns an array of arrays of Envelope, the outer arrays
    // representing SSKR groups and the inner array elements each holding the encrypted
    // seed and a single share.
    let content_key = SymmetricKey::new();
    let seed_envelope = dan_seed.enclose();
    let encrypted_seed_envelope = seed_envelope
        .encrypt_subject(&content_key).unwrap();

    let group = SSKRGroupSpec::new(2, 3).unwrap();
    let spec = SSKRSpec::new(1, vec![group]).unwrap();
    let envelopes = encrypted_seed_envelope
        .sskr_split(&spec, &content_key).unwrap();

    // Flattening the array of arrays gives just a single array of all the envelopes
    // to be distributed.
    let sent_envelopes: Vec<_> = envelopes.into_iter().flatten().collect();
    let sent_urs: Vec<_> = sent_envelopes.iter().map(|e| e.ur()).collect();

    let expected_format = indoc! {r#"
        ENCRYPTED [
            sskrShare: SSKRShare
        ]
        "#}.trim();
    assert_eq!(sent_envelopes[0].format(), expected_format);

    // Dan sends one envelope to each of Alice, Bob, and Carol.

    // Dan ➡️ ☁️ ➡️ Alice
    // Dan ➡️ ☁️ ➡️ Bob
    // Dan ➡️ ☁️ ➡️ Carol

    // let alice_envelope = Envelope::from_ur(&sent_urs[0]).unwrap(); // UNRECOVERED
    let bob_envelope = Rc::new(Envelope::from_ur(&sent_urs[1]).unwrap());
    let carol_envelope = Rc::new(Envelope::from_ur(&sent_urs[2]).unwrap());

    // At some future point, Dan retrieves two of the three envelopes so he can recover his seed.
    let recovered_envelopes = [bob_envelope.clone(), carol_envelope];
    let recovered_seed_envelope = Envelope::sskr_join(&recovered_envelopes).unwrap();
    println!("{}", recovered_seed_envelope.format());
    let recovered_seed = recovered_seed_envelope.extract_subject::<Seed>().unwrap();

    // The recovered seed is correct.
    assert_eq!(dan_seed.data(), recovered_seed.data());
    assert_eq!(dan_seed.creation_date(), recovered_seed.creation_date());
    assert_eq!(dan_seed.name(), recovered_seed.name());
    assert_eq!(dan_seed.note(), recovered_seed.note());

    // Attempting to recover with only one of the envelopes won't work.
    assert!(Envelope::sskr_join(&[bob_envelope]).is_err());
}
