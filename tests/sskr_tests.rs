#![cfg(feature = "sskr")]
use bc_components::{SymmetricKey, SSKRGroupSpec, SSKRSpec};
use hex_literal::hex;
use bc_envelope::prelude::*;
use indoc::indoc;

mod common;
use crate::common::test_seed::*;

#[test]
fn test_sskr() -> anyhow::Result<()> {
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
    let seed_envelope = dan_seed.envelope();
    let encrypted_seed_envelope = seed_envelope
        .wrap_envelope()
        .encrypt_subject(&content_key)?;

    let group = SSKRGroupSpec::new(2, 3)?;
    let spec = SSKRSpec::new(1, vec![group])?;
    let envelopes = encrypted_seed_envelope
        .sskr_split(&spec, &content_key)?;

    // Flattening the array of arrays gives just a single array of all the envelopes
    // to be distributed.
    let sent_envelopes: Vec<_> = envelopes.into_iter().flatten().collect();
    let sent_urs: Vec<_> = sent_envelopes.iter().map(|e| e.ur()).collect();

    let expected_format = indoc! {r#"
    ENCRYPTED [
        'sskrShare': SSKRShare
    ]
    "#}.trim();
    assert_eq!(sent_envelopes[0].format(), expected_format);

    // Dan sends one envelope to each of Alice, Bob, and Carol.

    // Dan ➡️ ☁️ ➡️ Alice
    // Dan ➡️ ☁️ ➡️ Bob
    // Dan ➡️ ☁️ ➡️ Carol

    // let alice_envelope = Envelope::from_ur(&sent_urs[0])?; // UNRECOVERED
    let bob_envelope = Envelope::from_ur(&sent_urs[1])?;
    let carol_envelope = Envelope::from_ur(&sent_urs[2])?;

    // At some future point, Dan retrieves two of the three envelopes so he can recover his seed.
    let recovered_envelopes = [bob_envelope.clone(), carol_envelope];
    let recovered_seed_envelope = Envelope::sskr_join(&recovered_envelopes)?.unwrap_envelope()?;

    let recovered_seed = Seed::from_envelope(&recovered_seed_envelope)?;

    // The recovered seed is correct.
    assert_eq!(dan_seed.data(), recovered_seed.data());
    assert_eq!(dan_seed.creation_date(), recovered_seed.creation_date());
    assert_eq!(dan_seed.name(), recovered_seed.name());
    assert_eq!(dan_seed.note(), recovered_seed.note());

    // Attempting to recover with only one of the envelopes won't work.
    assert!(Envelope::sskr_join(&[bob_envelope]).is_err());

    Ok(())
}
