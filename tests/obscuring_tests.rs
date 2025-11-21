#[cfg(feature = "encrypt")]
use bc_components::SymmetricKey;
use bc_envelope::prelude::*;

mod common;
use crate::common::test_data::*;

/// This tests the transformation of different kinds of "obscured" envelopes
/// into others. Some transformations are allowed, some are idempotent (return
/// the same result), and some throw errors.
///
/// | Operation > | Encrypt | Elide      | Compress   |
/// |:------------|:--------|:-----------|:-----------|
/// | Encrypted   | ERROR   | OK         | ERROR      |
/// | Elided      | ERROR   | IDEMPOTENT | ERROR      |
/// | Compressed  | OK      | OK         | IDEMPOTENT |
#[test]
fn test_obscuring() {
    #[cfg(feature = "encrypt")]
    let key = SymmetricKey::new();

    let envelope = Envelope::new(PLAINTEXT_HELLO);
    assert!(!envelope.is_obscured());

    #[cfg(feature = "encrypt")]
    {
        let encrypted = envelope.encrypt_subject(&key).unwrap();
        assert!(encrypted.is_obscured());
    }

    let elided = envelope.elide();
    assert!(elided.is_obscured());

    #[cfg(feature = "compress")]
    {
        let compressed = envelope.compress().unwrap();
        assert!(compressed.is_obscured());
    }
    // ENCRYPTION

    // Cannot encrypt an encrypted envelope.
    //
    // If allowed, would result in an envelope with the same digest but
    // double-encrypted, possibly with a different key, which is probably not
    // what's intended. If you want to double-encrypt then wrap the
    // encrypted envelope first, which will change its digest.
    #[cfg(feature = "encrypt")]
    {
        let encrypted = envelope.encrypt_subject(&key).unwrap();
        encrypted.encrypt_subject(&key).unwrap_err();
    }

    // Cannot encrypt an elided envelope.
    //
    // Elided envelopes have no data to encrypt.
    #[cfg(feature = "encrypt")]
    {
        elided.encrypt_subject(&key).unwrap_err();
    }

    #[cfg(all(feature = "compress", feature = "encrypt"))]
    {
        // OK to encrypt a compressed envelope.
        let compressed = envelope.compress().unwrap();
        let encrypted_compressed = compressed.encrypt_subject(&key).unwrap();
        assert!(encrypted_compressed.is_encrypted());
    }

    // ELISION

    #[cfg(feature = "encrypt")]
    {
        // OK to elide an encrypted envelope.
        let encrypted = envelope.encrypt_subject(&key).unwrap();
        let elided_encrypted = encrypted.elide();
        assert!(elided_encrypted.is_elided());
    }

    // Eliding an elided envelope is idempotent.
    let elided_elided = elided.elide();
    assert!(elided_elided.is_elided());

    #[cfg(feature = "compress")]
    {
        // OK to elide a compressed envelope.
        let compressed = envelope.compress().unwrap();
        let elided_compressed = compressed.elide();
        assert!(elided_compressed.is_elided());
    }

    // COMPRESSION

    // Cannot compress an encrypted envelope.
    //
    // Encrypted envelopes cannot become smaller because encrypted data looks
    // random, and random data is not compressible.
    #[cfg(all(feature = "compress", feature = "encrypt"))]
    {
        let encrypted = envelope.encrypt_subject(&key).unwrap();
        encrypted.compress().unwrap_err();
    }

    // Cannot compress an elided envelope.
    //
    // Elided envelopes have no data to compress.
    #[cfg(feature = "compress")]
    elided.compress().unwrap_err();

    #[cfg(feature = "compress")]
    {
        // Compressing a compressed envelope is idempotent.
        let compressed = envelope.compress().unwrap();
        let compressed_compressed = compressed.compress().unwrap();
        assert!(compressed_compressed.is_compressed());
    }
}

#[test]
fn test_nodes_matching() {
    use std::collections::HashSet;

    use indoc::indoc;

    let envelope = Envelope::new("Alice")
        .add_assertion("knows", "Bob")
        .add_assertion("age", 30)
        .add_assertion("city", "Boston");

    // Get some digests for targeting
    let knows_assertion = envelope.assertion_with_predicate("knows").unwrap();
    let knows_digest = knows_assertion.digest();

    #[cfg(feature = "compress")]
    let age_assertion = envelope.assertion_with_predicate("age").unwrap();
    #[cfg(feature = "compress")]
    let age_digest = age_assertion.digest();

    // Elide one assertion, compress another
    let mut elide_target = HashSet::new();
    elide_target.insert(knows_digest);

    #[cfg(feature = "compress")]
    let mut compress_target = HashSet::new();
    #[cfg(feature = "compress")]
    compress_target.insert(age_digest);

    #[cfg(feature = "compress")]
    let mut obscured = envelope.elide_removing_set(&elide_target);
    #[cfg(not(feature = "compress"))]
    let obscured = envelope.elide_removing_set(&elide_target);

    #[cfg(feature = "compress")]
    {
        obscured = obscured.elide_removing_set_with_action(
            &compress_target,
            &ObscureAction::Compress,
        );
    }

    // Verify the structure with elided and compressed nodes
    #[cfg(not(feature = "compress"))]
    #[rustfmt::skip]
    assert_eq!(obscured.format(), indoc! {r#"
        "Alice" [
            "age": 30
            "city": "Boston"
            ELIDED
        ]
    "#}.trim());

    #[cfg(feature = "compress")]
    #[rustfmt::skip]
    assert_eq!(obscured.format(), indoc! {r#"
        "Alice" [
            "city": "Boston"
            COMPRESSED
            ELIDED
        ]
    "#}.trim());

    // Test finding elided nodes
    let elided_nodes = obscured.nodes_matching(None, &[ObscureType::Elided]);
    assert!(elided_nodes.contains(&knows_digest));

    #[cfg(feature = "compress")]
    {
        // Test finding compressed nodes
        let compressed_nodes =
            obscured.nodes_matching(None, &[ObscureType::Compressed]);
        assert!(compressed_nodes.contains(&age_digest));
    }

    // Test finding with target filter
    let mut target_filter = HashSet::new();
    target_filter.insert(knows_digest);
    let filtered =
        obscured.nodes_matching(Some(&target_filter), &[ObscureType::Elided]);
    assert_eq!(filtered.len(), 1);
    assert!(filtered.contains(&knows_digest));

    // Test finding all obscured nodes (no type filter)
    let all_in_target = obscured.nodes_matching(Some(&elide_target), &[]);
    assert_eq!(all_in_target.len(), 1);
    assert!(all_in_target.contains(&knows_digest));

    // Test with no matches
    let mut no_match_target = HashSet::new();
    no_match_target.insert(Digest::from_image("nonexistent"));
    let no_matches =
        obscured.nodes_matching(Some(&no_match_target), &[ObscureType::Elided]);
    assert!(no_matches.is_empty());
}

#[test]
fn test_walk_unelide() {
    use indoc::indoc;

    let alice = Envelope::new("Alice");
    let bob = Envelope::new("Bob");
    let carol = Envelope::new("Carol");

    let envelope = Envelope::new("Alice")
        .add_assertion("knows", "Bob")
        .add_assertion("friend", "Carol");

    // Elide multiple parts
    let elided = envelope
        .elide_removing_target(&alice)
        .elide_removing_target(&bob);

    // Verify parts are elided
    #[rustfmt::skip]
    assert_eq!(elided.format(), indoc! {r#"
        ELIDED [
            "friend": "Carol"
            "knows": ELIDED
        ]
    "#}.trim());

    // Restore with walk_unelide
    let restored =
        elided.walk_unelide(&[alice.clone(), bob.clone(), carol.clone()]);

    // The restored envelope should match original
    #[rustfmt::skip]
    assert_eq!(restored.format(), indoc! {r#"
        "Alice" [
            "friend": "Carol"
            "knows": "Bob"
        ]
    "#}.trim());

    // Test with partial restoration (only some envelopes provided)
    let partial = elided.walk_unelide(std::slice::from_ref(&alice));
    #[rustfmt::skip]
    assert_eq!(partial.format(), indoc! {r#"
        "Alice" [
            "friend": "Carol"
            "knows": ELIDED
        ]
    "#}.trim());

    // Test with no matching envelopes
    let unchanged = elided.walk_unelide(&[]);
    assert!(unchanged.is_identical_to(&elided));
}

#[test]
#[cfg(feature = "encrypt")]
fn test_walk_decrypt() {
    use std::collections::HashSet;

    use bc_components::SymmetricKey;
    use indoc::indoc;

    let key1 = SymmetricKey::new();
    let key2 = SymmetricKey::new();
    let key3 = SymmetricKey::new();

    let envelope = Envelope::new("Alice")
        .add_assertion("knows", "Bob")
        .add_assertion("age", 30)
        .add_assertion("city", "Boston");

    // Encrypt different parts with different keys
    let knows_assertion = envelope.assertion_with_predicate("knows").unwrap();
    let age_assertion = envelope.assertion_with_predicate("age").unwrap();

    let mut encrypt1_target = HashSet::new();
    encrypt1_target.insert(knows_assertion.digest());

    let mut encrypt2_target = HashSet::new();
    encrypt2_target.insert(age_assertion.digest());

    let encrypted = envelope
        .elide_removing_set_with_action(
            &encrypt1_target,
            &ObscureAction::Encrypt(key1.clone()),
        )
        .elide_removing_set_with_action(
            &encrypt2_target,
            &ObscureAction::Encrypt(key2.clone()),
        );

    // Verify parts are encrypted
    #[rustfmt::skip]
    assert_eq!(encrypted.format(), indoc! {r#"
        "Alice" [
            "city": "Boston"
            ENCRYPTED (2)
        ]
    "#}.trim());

    // Decrypt with all keys
    let decrypted = encrypted.walk_decrypt(&[key1.clone(), key2.clone()]);
    #[rustfmt::skip]
    assert_eq!(decrypted.format(), indoc! {r#"
        "Alice" [
            "age": 30
            "city": "Boston"
            "knows": "Bob"
        ]
    "#}.trim());

    // Decrypt with only one key (partial decryption)
    let partial = encrypted.walk_decrypt(std::slice::from_ref(&key1));
    assert!(!partial.is_identical_to(&encrypted));
    // Note: partial is still equivalent because encrypted nodes preserve
    // digests
    assert!(partial.is_equivalent_to(&envelope));

    // There should still be one encrypted node remaining
    #[rustfmt::skip]
    assert_eq!(partial.format(), indoc! {r#"
        "Alice" [
            "city": "Boston"
            "knows": "Bob"
            ENCRYPTED
        ]
    "#}.trim());

    // Decrypt with wrong key (should be unchanged)
    let unchanged = encrypted.walk_decrypt(&[key3]);
    assert!(unchanged.is_identical_to(&encrypted));
}

#[test]
#[cfg(feature = "compress")]
fn test_walk_decompress() {
    use std::collections::HashSet;

    use indoc::indoc;

    let envelope = Envelope::new("Alice")
        .add_assertion("knows", "Bob")
        .add_assertion("bio", "A".repeat(1000))
        .add_assertion("description", "B".repeat(1000));

    // Compress multiple parts
    let bio_assertion = envelope.assertion_with_predicate("bio").unwrap();
    let desc_assertion =
        envelope.assertion_with_predicate("description").unwrap();

    let bio_digest = bio_assertion.digest();
    let desc_digest = desc_assertion.digest();

    let mut compress_target = HashSet::new();
    compress_target.insert(bio_digest);
    compress_target.insert(desc_digest);

    let compressed = envelope.elide_removing_set_with_action(
        &compress_target,
        &ObscureAction::Compress,
    );

    // Verify parts are compressed
    #[rustfmt::skip]
    assert_eq!(compressed.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob"
            COMPRESSED (2)
        ]
    "#}.trim());

    // decompress all
    let decompressed = compressed.walk_decompress(None);
    assert!(decompressed.is_equivalent_to(&envelope));

    // Decompress with target filter (only one node)
    let mut target = HashSet::new();
    target.insert(bio_digest);

    let partial = compressed.walk_decompress(Some(&target));
    assert!(!partial.is_identical_to(&compressed));
    // Note: partial is still equivalent because compressed nodes preserve
    // digests
    assert!(partial.is_equivalent_to(&envelope));

    // Bio should be decompressed but description still compressed
    let still_compressed =
        partial.nodes_matching(None, &[ObscureType::Compressed]);
    assert!(still_compressed.contains(&desc_digest));
    assert!(!still_compressed.contains(&bio_digest));

    // Decompress with non-matching target (should be unchanged)
    let mut no_match = HashSet::new();
    no_match.insert(Digest::from_image("nonexistent"));
    let unchanged = compressed.walk_decompress(Some(&no_match));
    assert!(unchanged.is_identical_to(&compressed));
}

#[test]
#[cfg(all(feature = "encrypt", feature = "compress"))]
fn test_mixed_obscuration_operations() {
    use std::collections::HashSet;

    use bc_components::SymmetricKey;

    let key = SymmetricKey::new();

    let envelope = Envelope::new("Alice")
        .add_assertion("knows", "Bob")
        .add_assertion("age", 30)
        .add_assertion("bio", "A".repeat(1000));

    let knows_assertion = envelope.assertion_with_predicate("knows").unwrap();
    let age_assertion = envelope.assertion_with_predicate("age").unwrap();
    let bio_assertion = envelope.assertion_with_predicate("bio").unwrap();

    let knows_digest = knows_assertion.digest();
    let age_digest = age_assertion.digest();
    let bio_digest = bio_assertion.digest();

    // Apply different obscuration types
    let mut elide_target = HashSet::new();
    elide_target.insert(knows_digest);

    let mut encrypt_target = HashSet::new();
    encrypt_target.insert(age_digest);

    let mut compress_target = HashSet::new();
    compress_target.insert(bio_digest);

    let obscured = envelope
        .elide_removing_set(&elide_target)
        .elide_removing_set_with_action(
            &encrypt_target,
            &ObscureAction::Encrypt(key.clone()),
        )
        .elide_removing_set_with_action(
            &compress_target,
            &ObscureAction::Compress,
        );

    // Verify different obscuration types
    let elided = obscured.nodes_matching(None, &[ObscureType::Elided]);
    let encrypted = obscured.nodes_matching(None, &[ObscureType::Encrypted]);
    let compressed = obscured.nodes_matching(None, &[ObscureType::Compressed]);

    assert!(elided.contains(&knows_digest));
    assert!(encrypted.contains(&age_digest));
    assert!(compressed.contains(&bio_digest));

    // Restore everything
    let restored = obscured
        .walk_unelide(std::slice::from_ref(&knows_assertion))
        .walk_decrypt(&[key])
        .walk_decompress(None);

    assert!(restored.is_equivalent_to(&envelope));
}
