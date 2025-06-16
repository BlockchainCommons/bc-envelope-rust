mod common;

use bc_envelope::prelude::*;
use bc_envelope_pattern::{Matcher, Pattern};
use indoc::indoc;

use crate::common::pattern_utils::*;

#[test]
fn test_subject_pattern() {
    let envelope = Envelope::new("Alice");

    let pat = Pattern::subject();
    let matching_paths = pat.paths(&envelope);

    #[rustfmt::skip]
    let expected = indoc! {r#"
        13941b48 "Alice"
    "#}.trim();
    assert_actual_expected!(format_paths(&matching_paths), expected);

    let envelope_with_assertions = envelope.add_assertion("knows", "Bob");
    let matching_paths = pat.paths(&envelope_with_assertions);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        8955db5e "Alice" [ "knows": "Bob" ]
            13941b48 "Alice"
    "#}.trim();
    assert_actual_expected!(format_paths(&matching_paths), expected);
}

#[test]
fn test_wrapped_pattern() {
    // Does not match non-wrapped subjects.
    let envelope = Envelope::new(42);
    assert!(!Pattern::wrapped().matches(&envelope));

    // Matches a wrapped envelope with any subject.
    let wrapped_envelope = envelope.wrap_envelope();
    let paths = Pattern::wrapped().paths(&wrapped_envelope);
    // println!("{}", format_paths(&paths));
    let expected = indoc! {r#"
        58b1ac6a { 42 }
            7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // The matched paths include the assertion.
    let wrapped_envelope_with_assertion = wrapped_envelope.add_assertion("an", "assertion");
    let paths = Pattern::wrapped().paths(&wrapped_envelope_with_assertion);
    // println!("{}", format_paths(&paths));
    let expected = indoc! {r#"
        169aba00 { 42 } [ "an": "assertion" ]
            7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    let wrapped_twice = wrapped_envelope_with_assertion.wrap_envelope();
    // Matching a wrapped envelope with assertions returns a path where the first
    // element is the original wrapped envelope including assertions, and the
    // second element is the still-wrapped subject.
    let paths = Pattern::wrapped().paths(&wrapped_twice);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        52d47c15 { { 42 } [ "an": "assertion" ] }
            169aba00 { 42 } [ "an": "assertion" ]
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    let wrapped_twice_pattern = Pattern::sequence(vec![
        Pattern::wrapped(),
        Pattern::wrapped(),
    ]);
    let paths = wrapped_twice_pattern.paths(&wrapped_twice);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        52d47c15 { { 42 } [ "an": "assertion" ] }
            169aba00 { 42 } [ "an": "assertion" ]
                7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn test_assertion_pattern() {
    let envelope_without_assertions = Envelope::new("Alice");

    // Does not match envelopes without assertions.
    assert!(!Pattern::any_assertion().matches(&envelope_without_assertions));

    let envelope_with_assertions = envelope_without_assertions
        .add_assertion("knows", "Bob")
        .add_assertion("worksWith", "Charlie");

    // Returns a path for each assertion in the envelope.
    let paths = Pattern::any_assertion().paths(&envelope_with_assertions);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        78d666eb "knows": "Bob"
        c269cf0a "worksWith": "Charlie"
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn test_assertion_predicate_pattern() {
    let envelope = Envelope::new("Alice")
        .add_assertion("knows", "Bob")
        .add_assertion("knows", "Charlie")
        .add_assertion("worksWith", "David");

    // Returns a path for each assertion with a predicate that matches
    // the specified pattern.
    let paths = Pattern::assertion_with_predicate(Pattern::text("knows"))
        .paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        78d666eb "knows": "Bob"
        7af83724 "knows": "Charlie"
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    let pattern = Pattern::sequence(vec![
        Pattern::assertion_with_predicate(Pattern::text("knows")),
        Pattern::any_object(),
    ]);
    let paths = pattern.paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        78d666eb "knows": "Bob"
            13b74194 "Bob"
        7af83724 "knows": "Charlie"
            ee8e3b02 "Charlie"
    "#}
    .trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn test_digest_pattern() {
    let envelope = Envelope::new("Hello, World!");
    let digest = envelope.digest().into_owned();
    let hex_digest = hex::encode(digest.as_bytes());
    let prefix = &hex_digest[0..8];

    // Test exact digest matching
    assert!(Pattern::digest(digest.clone()).matches(&envelope));
    assert!(!Pattern::digest(Digest::from_data([0; 32])).matches(&envelope));

    // Test hex prefix matching
    assert!(Pattern::digest_hex_prefix(prefix).matches(&envelope));
    assert!(Pattern::digest_hex_prefix(&hex_digest).matches(&envelope));
    assert!(!Pattern::digest_hex_prefix("ffffffff").matches(&envelope));

    // Test with envelope that has assertions
    let envelope_with_assertions =
        envelope.clone().add_assertion("test", "value");
    let digest_with_assertions = envelope_with_assertions.digest().into_owned();

    assert!(
        !Pattern::digest(digest.clone()).matches(&envelope_with_assertions)
    );
    assert!(
        Pattern::digest(digest_with_assertions.clone())
            .matches(&envelope_with_assertions)
    );

    // Test paths
    let paths = Pattern::digest(digest.clone()).paths(&envelope);
    #[rustfmt::skip]
    let expected = format!(
        "{} \"Hello, World!\"",
        envelope.short_id(DigestDisplayFormat::Short)
    );
    assert_actual_expected!(format_paths(&paths), expected);

    // No match should return empty paths
    let paths = Pattern::digest(Digest::from_data([0; 32])).paths(&envelope);
    assert!(paths.is_empty());
}

#[test]
fn test_digest_pattern_binary_regex() {
    let envelope = Envelope::new("Hello, World!");
    let digest = envelope.digest().into_owned();
    let digest_bytes = digest.data();

    // IMPORTANT: Binary regex patterns must use the (?s-u) flags to work
    // correctly with arbitrary bytes:
    // - (?s) allows '.' to match newline characters (like 0x0a)
    // - (?-u) disables unicode mode so '.' matches any byte, not just valid
    //   UTF-8 sequences
    // Without these flags, patterns like ^.{32}$ will fail on digest bytes
    // containing newlines or invalid UTF-8 sequences (which are common in
    // SHA-256 digests).

    // Test regex matching any 32 bytes (should match any valid digest)
    // NOTE: Need (?s-u) flags to handle all bytes including newlines and
    // invalid UTF-8
    let any_32_bytes_regex =
        regex::bytes::Regex::new(r"(?s-u)^.{32}$").unwrap();
    assert!(
        Pattern::digest_binary_regex(any_32_bytes_regex).matches(&envelope)
    );

    // Test regex that shouldn't match (all zeros, 32 bytes)
    let all_zeros_regex =
        regex::bytes::Regex::new(r"(?s-u)^\x00{32}$").unwrap();
    assert!(!Pattern::digest_binary_regex(all_zeros_regex).matches(&envelope));

    // Test regex matching specific byte values using hex escapes
    let first_byte_hex = format!(r"(?s-u)^\x{:02x}", digest_bytes[0]);
    let starts_with_first_byte =
        regex::bytes::Regex::new(&first_byte_hex).unwrap();
    assert!(
        Pattern::digest_binary_regex(starts_with_first_byte).matches(&envelope)
    );

    // Test regex for exact first two bytes
    let first_two_bytes_pattern =
        format!(r"(?s-u)^\x{:02x}\x{:02x}", digest_bytes[0], digest_bytes[1]);
    let exact_first_two =
        regex::bytes::Regex::new(&first_two_bytes_pattern).unwrap();
    assert!(Pattern::digest_binary_regex(exact_first_two).matches(&envelope));

    // Test regex matching any byte in range (should match digests with varied
    // byte values)
    let any_byte_range =
        regex::bytes::Regex::new(r"(?s-u)[\x00-\xFF]").unwrap();
    assert!(Pattern::digest_binary_regex(any_byte_range).matches(&envelope));

    // Test complex pattern - match digests that start with a non-zero byte
    let non_zero_start =
        regex::bytes::Regex::new(r"(?s-u)^[\x01-\xFF]").unwrap();
    let should_match = digest_bytes[0] != 0;
    assert_eq!(
        Pattern::digest_binary_regex(non_zero_start).matches(&envelope),
        should_match
    );

    // Test paths for matching regex
    let paths = Pattern::digest_binary_regex(
        regex::bytes::Regex::new(r"(?s-u)^.{32}$").unwrap(),
    )
    .paths(&envelope);
    #[rustfmt::skip]
    let expected = format!(
        "{} \"Hello, World!\"",
        envelope.short_id(DigestDisplayFormat::Short)
    );
    assert_actual_expected!(format_paths(&paths), expected);

    // Test paths for non-matching regex
    let paths = Pattern::digest_binary_regex(
        regex::bytes::Regex::new(r"(?s-u)^\x00{32}$").unwrap(),
    )
    .paths(&envelope);
    assert!(paths.is_empty());

    // Test with different envelope to ensure digest changes
    let envelope2 = Envelope::new("Different content");
    let digest2 = envelope2.digest().into_owned();

    // Verify the digests are actually different
    assert_ne!(digest.data(), digest2.data());

    // Test a pattern that matches both (any 32 bytes)
    let any_digest_regex = regex::bytes::Regex::new(r"(?s-u)^.{32}$").unwrap();
    assert!(
        Pattern::digest_binary_regex(any_digest_regex.clone())
            .matches(&envelope)
    );
    assert!(Pattern::digest_binary_regex(any_digest_regex).matches(&envelope2));

    // Test pattern that matches based on content
    // Looking for digests containing specific byte sequences
    let contains_aa = regex::bytes::Regex::new(r"(?s-u)\xaa").unwrap();
    let envelope1_has_aa = digest_bytes.contains(&0xaa);
    assert_eq!(
        Pattern::digest_binary_regex(contains_aa.clone()).matches(&envelope),
        envelope1_has_aa
    );

    let digest2_bytes = digest2.data();
    let envelope2_has_aa = digest2_bytes.contains(&0xaa);
    assert_eq!(
        Pattern::digest_binary_regex(contains_aa).matches(&envelope2),
        envelope2_has_aa
    );

    // Test edge cases with different digest patterns
    let pattern_32_ascending =
        regex::bytes::Regex::new(r"(?s-u)^[\x00-\x1F]{32}$").unwrap();
    let has_low_bytes = digest_bytes.iter().all(|&b| b <= 0x1F);
    assert_eq!(
        Pattern::digest_binary_regex(pattern_32_ascending).matches(&envelope),
        has_low_bytes
    );

    // Test pattern that looks for high-bit bytes
    let pattern_high_bit =
        regex::bytes::Regex::new(r"(?s-u)[\x80-\xFF]").unwrap();
    let has_high_bit = digest_bytes.iter().any(|&b| b >= 0x80);
    assert_eq!(
        Pattern::digest_binary_regex(pattern_high_bit).matches(&envelope),
        has_high_bit
    );

    // Test envelope with assertions (should match differently)
    let envelope_with_assertions =
        envelope.clone().add_assertion("test", "value");
    let digest_with_assertions = envelope_with_assertions.digest().into_owned();

    // Should still match 32-byte pattern
    let universal_pattern = regex::bytes::Regex::new(r"(?s-u)^.{32}$").unwrap();
    assert!(
        Pattern::digest_binary_regex(universal_pattern)
            .matches(&envelope_with_assertions)
    );

    // But digests should be different
    assert_ne!(digest.data(), digest_with_assertions.data());
}

#[test]
fn test_node_pattern() {
    // Test with leaf (non-node) envelope
    let leaf_envelope = Envelope::new("Just a leaf");
    assert!(!Pattern::any_node().matches(&leaf_envelope));
    assert!(
        !Pattern::node_with_assertions_count_range(1..=3)
            .matches(&leaf_envelope)
    );
    assert!(!Pattern::node_with_assertions_count(1).matches(&leaf_envelope));

    // Test with single assertion node
    let single_assertion_envelope =
        Envelope::new("Alice").add_assertion("knows", "Bob");
    assert!(Pattern::any_node().matches(&single_assertion_envelope));
    assert!(
        Pattern::node_with_assertions_count_range(1..=3)
            .matches(&single_assertion_envelope)
    );
    assert!(
        Pattern::node_with_assertions_count(1)
            .matches(&single_assertion_envelope)
    );
    assert!(
        !Pattern::node_with_assertions_count(2)
            .matches(&single_assertion_envelope)
    );

    // Test with multiple assertions node
    let multi_assertion_envelope = single_assertion_envelope
        .add_assertion("age", 25)
        .add_assertion("city", "New York");
    assert!(Pattern::any_node().matches(&multi_assertion_envelope));
    assert!(
        Pattern::node_with_assertions_count_range(1..=5)
            .matches(&multi_assertion_envelope)
    );
    assert!(
        Pattern::node_with_assertions_count(3)
            .matches(&multi_assertion_envelope)
    );
    assert!(
        !Pattern::node_with_assertions_count(2)
            .matches(&multi_assertion_envelope)
    );
    assert!(
        !Pattern::node_with_assertions_count_range(4..=5)
            .matches(&multi_assertion_envelope)
    );

    // Test paths
    let paths = Pattern::any_node().paths(&single_assertion_envelope);
    #[rustfmt::skip]
    let expected = format!(
        r#"{} "Alice" [ "knows": "Bob" ]"#,
        single_assertion_envelope.short_id(DigestDisplayFormat::Short)
    );
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn test_obscured_pattern() {
    let original_envelope = Envelope::new("Secret data");

    // Test with elided envelope
    let elided_envelope = original_envelope.elide();
    assert!(Pattern::obscured().matches(&elided_envelope));
    assert!(Pattern::elided().matches(&elided_envelope));
    assert!(!Pattern::encrypted().matches(&elided_envelope));
    assert!(!Pattern::compressed().matches(&elided_envelope));

    // Test with original (non-obscured) envelope
    assert!(!Pattern::obscured().matches(&original_envelope));
    assert!(!Pattern::elided().matches(&original_envelope));
    assert!(!Pattern::encrypted().matches(&original_envelope));
    assert!(!Pattern::compressed().matches(&original_envelope));

    // Test paths for elided envelope
    let paths = Pattern::elided().paths(&elided_envelope);
    #[rustfmt::skip]
    let expected = format!(
        "{} ELIDED",
        elided_envelope.short_id(DigestDisplayFormat::Short)
    );
    assert_actual_expected!(format_paths(&paths), expected);

    // No match should return empty paths
    let paths = Pattern::elided().paths(&original_envelope);
    assert!(paths.is_empty());

    #[cfg(feature = "encrypt")]
    {
        use bc_components::SymmetricKey;

        // Test with encrypted envelope
        let key = SymmetricKey::new();
        let encrypted_envelope =
            original_envelope.encrypt_subject(&key).unwrap();
        assert!(Pattern::obscured().matches(&encrypted_envelope));
        assert!(Pattern::encrypted().matches(&encrypted_envelope));
        assert!(!Pattern::elided().matches(&encrypted_envelope));
        assert!(!Pattern::compressed().matches(&encrypted_envelope));
    }

    #[cfg(feature = "compress")]
    {
        // Test with compressed envelope
        let compressed_envelope = original_envelope.compress().unwrap();
        assert!(Pattern::obscured().matches(&compressed_envelope));
        assert!(Pattern::compressed().matches(&compressed_envelope));
        assert!(!Pattern::elided().matches(&compressed_envelope));
        assert!(!Pattern::encrypted().matches(&compressed_envelope));
    }
}
