#![cfg(all(
    feature = "signature",
    feature = "sskr",
    feature = "secret",
    feature = "types"
))]

mod common;

use bc_components::{
    KeyDerivationMethod, SSKRGroupSpec, SSKRSpec, SymmetricKey, keypair,
};
use bc_envelope::prelude::*;
use dcbor::Date;
use indoc::indoc;

#[test]
fn test_multi_permit() -> anyhow::Result<()> {
    bc_components::register_tags();

    //
    // Alice composes a poem.
    //
    let poem_text =
        "At midnight, the clocks sang lullabies to the wandering teacups.";

    //
    // Alice creates a new envelope and assigns the text as the envelope's
    // subject. She also adds some metadata assertions to the envelope,
    // including that the subject is a "poem", the title, the author, and
    // the date.
    //
    let original_envelope = Envelope::new(poem_text)
        .add_type("poem")
        .add_assertion("title", "A Song of Ice Cream")
        .add_assertion("author", "Plonkus the Iridescent")
        .add_assertion(known_values::DATE, Date::from_ymd(2025, 5, 15));

    //
    // Alice signs the envelope with her private key.
    //
    let (alice_private_keys, alice_public_keys) = keypair();
    let signed_envelope = original_envelope.sign(&alice_private_keys);
    #[rustfmt::skip]
    let expected_format = indoc! {r#"
        {
            "At midnight, the clocks sang lullabies to the wandering teacups." [
                'isA': "poem"
                "author": "Plonkus the Iridescent"
                "title": "A Song of Ice Cream"
                'date': 2025-05-15
            ]
        } [
            'signed': Signature
        ]
    "#}.trim();
    assert_actual_expected!(signed_envelope.format(), expected_format);

    //
    // Alice picks a random symmetric "content key" and uses it to encrypt the
    // signed envelope. She will provide several different methods ("permits")
    // that can be used to unlock it. Each permit encrypts the same content key
    // using a different method.
    //
    let content_key = SymmetricKey::new();
    let encrypted_envelope = signed_envelope.encrypt(&content_key);
    #[rustfmt::skip]
    let expected_format = indoc! {r#"
        ENCRYPTED
    "#}.trim();
    assert_actual_expected!(encrypted_envelope.format(), expected_format);

    //
    // Alice wants to be able to recover the envelope later using a password she
    // can remember, So she adds the first permit to the envelope by using the
    // `add_secret()` method, providing a derivation method `Argon2id`, her
    // password, and the content key. The `add_secret()` method encrypts the
    // content key with a key derived from her password, and adds it to the
    // envelope as a `'hasSecret'` assertion.
    //
    let password = b"unicorns_dance_on_mars_while_eating_pizza";
    let locked_envelope = encrypted_envelope
        .add_secret(KeyDerivationMethod::Argon2id, &password, &content_key)
        .unwrap();
    // println!("{}", locked_envelope.format());
    #[rustfmt::skip]
    let expected_format = indoc! {r#"
        ENCRYPTED [
            'hasSecret': EncryptedKey(Argon2id)
        ]
    "#}.trim();
    assert_actual_expected!(locked_envelope.format(), expected_format);

    //
    // Next, Alice wants to be able to unlock her envelope using her private
    // key, and she also wants Bob to be able to unlock it using his private
    // key. To do this, she uses the `add_recipient()` method, which
    // encrypts the content key with the public keys of Alice and Bob.
    let (bob_private_keys, bob_public_keys) = keypair();
    let locked_envelope = locked_envelope
        .add_recipient(&alice_public_keys, &content_key)
        .add_recipient(&bob_public_keys, &content_key);
    // println!("{}", locked_envelope.format());
    #[rustfmt::skip]
    let expected_format = indoc! {r#"
        ENCRYPTED [
            'hasRecipient': SealedMessage
            'hasRecipient': SealedMessage
            'hasSecret': EncryptedKey(Argon2id)
        ]
    "#}.trim();
    assert_actual_expected!(locked_envelope.format(), expected_format);

    //
    // An SSKR share is a kind of permit defined by the characteristic that one
    // share by itself is not enough to unlock the envelope: some quorum of
    // shares is required.
    //
    // Alice wants to back up her poem using a social recovery scheme, So even
    // if she forgets her password and loses her private key, she can still
    // recover the envelope by finding two of the three friends she entrusted
    // with the shares.
    //
    // So Alice creates a 2-of-3 SSKR group and "shards" the envelope into three
    // envelopes, each containing a unique SSKR share.
    //
    let sskr_group = SSKRGroupSpec::new(2, 3)?;
    let spec = SSKRSpec::new(1, vec![sskr_group])?;
    let sharded_envelopes =
        locked_envelope.sskr_split_flattened(&spec, &content_key)?;

    //
    // Every envelope looks the same including the previous permits Alice added,
    // but each one contains a different SSKR share, so we only show the first
    // one here.
    //
    // println!("{}", sharded_envelopes[0].format());
    #[rustfmt::skip]
    let expected_format = indoc! {r#"
        ENCRYPTED [
            'hasRecipient': SealedMessage
            'hasRecipient': SealedMessage
            'hasSecret': EncryptedKey(Argon2id)
            'sskrShare': SSKRShare
        ]
    "#}.trim();
    assert_actual_expected!(sharded_envelopes[0].format(), expected_format);

    //
    // So now there are three envelopes, and five different ways to unlock
    // them:
    //
    // 1. Using her original content key (usually not saved, but could be stored
    //    in a safe place)
    // 2. Using her password
    // 3. Using her private key
    // 4. Using Bob's private key
    // 5. Using any two of the three SSKR shares
    //

    //
    // Using the content key.
    //

    let received_envelope = &sharded_envelopes[0];
    let unlocked_envelope = received_envelope.decrypt(&content_key)?;
    assert_eq!(unlocked_envelope, signed_envelope);

    //
    // Using the password and the Argon2id method.
    //

    let unlocked_envelope = received_envelope.unlock(&password)?;
    assert_eq!(unlocked_envelope, signed_envelope);

    //
    // Using Alice's private key.
    //

    let unlocked_envelope =
        received_envelope.decrypt_to_recipient(&alice_private_keys)?;
    assert_eq!(unlocked_envelope, signed_envelope);

    //
    // Using Bob's private key.
    //

    let unlocked_envelope =
        received_envelope.decrypt_to_recipient(&bob_private_keys)?;
    assert_eq!(unlocked_envelope, signed_envelope);

    //
    // Using any two of the three SSKR shares.
    //

    let unlocked_envelope =
        Envelope::sskr_join(&[&sharded_envelopes[0], &sharded_envelopes[2]])?
            .unwrap_envelope()?;
    // println!("{}", unlocked_envelope.format());
    assert_eq!(unlocked_envelope, signed_envelope);

    unlocked_envelope.verify(&alice_public_keys)?;

    Ok(())
}
