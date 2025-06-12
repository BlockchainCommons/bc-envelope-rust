mod common;

use bc_envelope::prelude::*;
use indoc::indoc;

use crate::common::pattern_utils::*;

#[test]
fn test_bool_pattern() {
    // Does not match non-boolean subjects.
    let envelope = Envelope::new(42);
    assert!(!Pattern::any_bool().matches(&envelope));
    assert!(!Pattern::bool(true).matches(&envelope));
    assert!(!Pattern::bool(false).matches(&envelope));

    // Matches a bare subject that is a boolean.
    let envelope = Envelope::new(true);
    assert!(Pattern::any_bool().matches(&envelope));
    assert!(Pattern::bool(true).matches(&envelope));
    assert!(!Pattern::bool(false).matches(&envelope));

    // Matches a subject that is a boolean with an assertion.
    let envelope = envelope.add_assertion("an", "assertion");
    assert!(Pattern::any_bool().matches(&envelope));
    assert!(Pattern::bool(true).matches(&envelope));
    assert!(!Pattern::bool(false).matches(&envelope));

    // The matched paths include the assertion. In other words, the
    // path just includes the envelope itself as its only element.
    let paths = Pattern::any_bool().paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        50d1019e true [ "an": "assertion" ]
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // Matching a boolean and the subject in a sequence returns a path
    // where the first element is the original envelope and the second
    // element is the subject.
    let paths =
        Pattern::sequence(vec![Pattern::any_bool(), Pattern::subject()])
            .paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        50d1019e true [ "an": "assertion" ]
            27abdedd true
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn test_number_pattern() {
    // Does not match non-number subjects.
    let envelope = Envelope::new("string");
    assert!(!Pattern::any_number().matches(&envelope));
    assert!(!Pattern::number(42).matches(&envelope));

    // Matches a bare subject that is a number.
    let envelope = Envelope::new(42);
    assert!(Pattern::any_number().matches(&envelope));
    assert!(Pattern::number(42).matches(&envelope));
    assert!(!Pattern::number(43).matches(&envelope));
    assert!(Pattern::number_range(40..=50).matches(&envelope));
    assert!(!Pattern::number_range(43..=50).matches(&envelope));
    assert!(Pattern::number_greater_than(41).matches(&envelope));
    assert!(!Pattern::number_greater_than(42).matches(&envelope));
    assert!(Pattern::number_less_than(43).matches(&envelope));
    assert!(!Pattern::number_less_than(42).matches(&envelope));
    assert!(Pattern::number_greater_than_or_equal(42).matches(&envelope));
    assert!(!Pattern::number_greater_than_or_equal(43).matches(&envelope));

    // Matches a subject that is a number with an assertion.
    let envelope = envelope.add_assertion("an", "assertion");
    assert!(Pattern::any_number().matches(&envelope));
    assert!(Pattern::number(42).matches(&envelope));

    // The matched paths include the assertion.
    let paths = Pattern::any_number().paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        6cb2ea4a 42 [ "an": "assertion" ]
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // Matching a number and the subject in a sequence returns a path
    // where the first element is the original envelope and the second
    // element is the subject.
    let paths =
        Pattern::sequence(vec![Pattern::any_number(), Pattern::subject()])
            .paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        6cb2ea4a 42 [ "an": "assertion" ]
            7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn test_text_pattern() {
    // Does not match non-text subjects.
    let envelope = Envelope::new(42);
    assert!(!Pattern::any_text().matches(&envelope));
    assert!(!Pattern::text("hello").matches(&envelope));

    // Matches a bare subject that is text.
    let envelope = Envelope::new("hello");
    assert!(Pattern::any_text().matches(&envelope));
    assert!(Pattern::text("hello").matches(&envelope));
    assert!(!Pattern::text("world").matches(&envelope));
    let regex = regex::Regex::new(r"^h.*o$").unwrap();
    assert!(Pattern::text_regex(&regex).matches(&envelope));

    // Matches a subject that is text with an assertion.
    let envelope = envelope.add_assertion("greeting", "world");
    assert!(Pattern::any_text().matches(&envelope));
    assert!(Pattern::text("hello").matches(&envelope));
    assert!(!Pattern::text("world").matches(&envelope));
    assert!(Pattern::text_regex(&regex).matches(&envelope));

    // The matched paths include the assertion.
    let paths = Pattern::any_text().paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        80a8c700 "hello" [ "greeting": "world" ]
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // Matching a text and the subject in a sequence returns a path
    // where the first element is the original envelope and the second
    // element is the subject.
    let paths =
        Pattern::sequence(vec![Pattern::any_text(), Pattern::subject()])
            .paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        80a8c700 "hello" [ "greeting": "world" ]
            cb835593 "hello"
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[cfg(feature = "known_value")]
#[test]
fn test_known_value_pattern() {
    use known_values;

    // Does not match non-known-value subjects.
    let envelope = Envelope::new("test");
    assert!(!Pattern::any_known_value().matches(&envelope));
    assert!(!Pattern::known_value(known_values::DATE).matches(&envelope));
    assert!(!Pattern::known_value_named("date").matches(&envelope));

    // Matches a bare subject that is a known value.
    let envelope = Envelope::new(known_values::DATE);
    assert!(Pattern::any_known_value().matches(&envelope));
    assert!(Pattern::known_value(known_values::DATE).matches(&envelope));
    assert!(Pattern::known_value_named("date").matches(&envelope));
    assert!(!Pattern::known_value(known_values::LANGUAGE).matches(&envelope));
    assert!(!Pattern::known_value_named("language").matches(&envelope));

    // Matches a subject that is a known value with an assertion.
    let envelope = envelope.add_assertion("meaning", "timestamp");
    assert!(Pattern::any_known_value().matches(&envelope));
    assert!(Pattern::known_value(known_values::DATE).matches(&envelope));
    assert!(Pattern::known_value_named("date").matches(&envelope));
    assert!(!Pattern::known_value(known_values::LANGUAGE).matches(&envelope));
    assert!(!Pattern::known_value_named("language").matches(&envelope));

    // The matched paths include the assertion. In other words, the
    // path just includes the envelope itself as its only element.
    let paths = Pattern::any_known_value().paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        813f39cd 'date' [ "meaning": "timestamp" ]
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // Test matching by name
    let paths = Pattern::known_value_named("date").paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        813f39cd 'date' [ "meaning": "timestamp" ]
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // Matching a known value and the subject in a sequence returns a path
    // where the first element is the original envelope and the second
    // element is the subject.
    let paths =
        Pattern::sequence(vec![Pattern::any_known_value(), Pattern::subject()])
            .paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        813f39cd 'date' [ "meaning": "timestamp" ]
            2e40139b 'date'
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // Test with unknown name should not match
    assert!(!Pattern::known_value_named("unknown_name").matches(&envelope));
}

#[cfg(feature = "known_value")]
#[test]
fn test_known_value_regex_pattern() {
    use regex::Regex;

    let value = known_values::DATE;
    let envelope =
        Envelope::new(value.clone()).add_assertion("meaning", "timestamp");

    // Test regex that matches "date" - Pattern should match
    let regex = Regex::new(r"^da.*").unwrap();
    assert!(Pattern::known_value_regex(regex).matches(&envelope));

    // Test regex that matches names ending with "te" - Pattern should match
    let regex = Regex::new(r".*te$").unwrap();
    assert!(Pattern::known_value_regex(regex).matches(&envelope));

    // Test case-insensitive regex - Pattern should match
    let regex = Regex::new(r"(?i)^DATE$").unwrap();
    assert!(Pattern::known_value_regex(regex).matches(&envelope));

    // Test regex that doesn't match - Pattern should not match
    let regex = Regex::new(r"^lang.*").unwrap();
    assert!(!Pattern::known_value_regex(regex).matches(&envelope));

    // Test with a different known value
    let language_value = known_values::LANGUAGE;
    let language_envelope = Envelope::new(language_value.clone());

    // This regex should match "language"
    let regex = Regex::new(r"lang.*").unwrap();
    assert!(Pattern::known_value_regex(regex).matches(&language_envelope));

    // This regex should not match "language"
    let regex = Regex::new(r"^da.*").unwrap();
    assert!(!Pattern::known_value_regex(regex).matches(&language_envelope));

    // Test with non-known-value envelope - Pattern should not match
    let text_envelope = Envelope::new("test");
    let regex = Regex::new(r".*").unwrap();
    assert!(!Pattern::known_value_regex(regex).matches(&text_envelope));

    // Test regex pattern in sequence patterns
    let regex = Regex::new(r".*te$").unwrap();
    let paths = Pattern::sequence(vec![
        Pattern::known_value_regex(regex),
        Pattern::subject(),
    ])
    .paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        813f39cd 'date' [ "meaning": "timestamp" ]
            2e40139b 'date'
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn test_byte_string_pattern() {
    use dcbor::prelude::*;

    // Does not match non-byte-string subjects.
    let envelope = Envelope::new("string");
    assert!(!Pattern::any_byte_string().matches(&envelope));
    assert!(!Pattern::byte_string(vec![1, 2, 3]).matches(&envelope));

    // Test with a byte string "Hello"
    let hello_bytes = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]; // "Hello"
    let envelope = Envelope::new(CBOR::to_byte_string(hello_bytes.clone()));

    // Matches any byte string
    assert!(Pattern::any_byte_string().matches(&envelope));

    // Matches exact byte string
    assert!(Pattern::byte_string(hello_bytes.clone()).matches(&envelope));
    assert!(!Pattern::byte_string(vec![1, 2, 3]).matches(&envelope));

    // Matches binary regex
    let regex = regex::bytes::Regex::new(r"^He.*o$").unwrap();
    assert!(Pattern::byte_string_binary_regex(regex).matches(&envelope));

    let non_matching_regex = regex::bytes::Regex::new(r"^World").unwrap();
    assert!(
        !Pattern::byte_string_binary_regex(non_matching_regex)
            .matches(&envelope)
    );

    // Matches a subject that is a byte string with an assertion.
    let envelope = envelope.add_assertion("type", "greeting");
    assert!(Pattern::any_byte_string().matches(&envelope));
    assert!(Pattern::byte_string(hello_bytes.clone()).matches(&envelope));

    let regex = regex::bytes::Regex::new(r".*llo.*").unwrap();
    assert!(Pattern::byte_string_binary_regex(regex).matches(&envelope));

    // The matched paths include the assertion.
    let paths = Pattern::any_byte_string().paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        19c2bef3 Bytes(5) [ "type": "greeting" ]
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // Matching a byte string and the subject in a sequence returns a path
    // where the first element is the original envelope and the second
    // element is the subject.
    let paths =
        Pattern::sequence(vec![Pattern::any_byte_string(), Pattern::subject()])
            .paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        19c2bef3 Bytes(5) [ "type": "greeting" ]
            3a91d2eb Bytes(5)
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // Test with binary regex patterns
    // IMPORTANT: Binary regex patterns must use the (?s-u) flags to work
    // correctly with arbitrary bytes:
    // - (?s) allows '.' to match newline characters (like 0x0a)
    // - (?-u) disables unicode mode so '.' matches any byte, not just valid
    //   UTF-8 sequences
    let binary_data = vec![0x00, 0x01, 0xFF, 0xAB, 0xCD];
    let binary_envelope =
        Envelope::new(CBOR::to_byte_string(binary_data.clone()));

    // Test pattern that matches any byte containing 0xFF
    let regex = regex::bytes::Regex::new(r"(?s-u).*\xFF.*").unwrap();
    assert!(Pattern::byte_string_binary_regex(regex).matches(&binary_envelope));

    // Test pattern that matches specific sequence
    let regex = regex::bytes::Regex::new(r"(?s-u)\x00\x01").unwrap();
    assert!(Pattern::byte_string_binary_regex(regex).matches(&binary_envelope));

    // Test pattern that matches any 5 bytes (should match our binary data)
    let regex = regex::bytes::Regex::new(r"(?s-u)^.{5}$").unwrap();
    assert!(Pattern::byte_string_binary_regex(regex).matches(&binary_envelope));

    // Test pattern that doesn't match (starts with 0xFF)
    let regex = regex::bytes::Regex::new(r"(?s-u)^\xFF").unwrap();
    assert!(
        !Pattern::byte_string_binary_regex(regex).matches(&binary_envelope)
    );

    // Test pattern matching high bit bytes
    let regex = regex::bytes::Regex::new(r"(?s-u)[\x80-\xFF]").unwrap();
    assert!(Pattern::byte_string_binary_regex(regex).matches(&binary_envelope));
}

#[test]
fn test_array_pattern() {
    use dcbor::prelude::*;

    // Does not match non-array subjects.
    let envelope = Envelope::new("string");
    assert!(!Pattern::any_array().matches(&envelope));
    assert!(!Pattern::array_count(3).matches(&envelope));

    // Test with a CBOR array
    let array_data = vec![1, 2, 3].to_cbor();
    let envelope = Envelope::new(array_data);

    // Matches any array
    assert!(Pattern::any_array().matches(&envelope));

    // Matches exact count
    assert!(Pattern::array_count(3).matches(&envelope));
    assert!(!Pattern::array_count(5).matches(&envelope));

    // Matches count range
    assert!(Pattern::array_count_range(2..=4).matches(&envelope));
    assert!(!Pattern::array_count_range(5..=10).matches(&envelope));

    // Test with assertions
    let envelope = envelope.add_assertion("type", "list");
    assert!(Pattern::any_array().matches(&envelope));
    assert!(Pattern::array_count(3).matches(&envelope));

    // The matched paths include the assertion
    let paths = Pattern::any_array().paths(&envelope);
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0], vec![envelope.clone()]);

    // Test with empty array
    let empty_array = Vec::<i32>::new().to_cbor();
    let empty_envelope = Envelope::new(empty_array);
    assert!(Pattern::any_array().matches(&empty_envelope));
    assert!(Pattern::array_count(0).matches(&empty_envelope));
    assert!(!Pattern::array_count(1).matches(&empty_envelope));

    // Test sequence patterns
    let paths =
        Pattern::sequence(vec![Pattern::any_array(), Pattern::subject()])
            .paths(&envelope);
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].len(), 2);
    assert_eq!(paths[0][0], envelope);
    assert_eq!(paths[0][1], envelope.subject());
}

#[test]
fn test_map_pattern() {
    use dcbor::prelude::*;

    // Does not match non-map subjects.
    let envelope = Envelope::new("string");
    assert!(!Pattern::any_map().matches(&envelope));
    assert!(!Pattern::map_count(2).matches(&envelope));

    // Test with a CBOR map
    let mut map = Map::new();
    map.insert("key1", "value1");
    map.insert("key2", "value2");
    let envelope = Envelope::new(map);

    // Matches any map
    assert!(Pattern::any_map().matches(&envelope));

    // Matches exact count
    assert!(Pattern::map_count(2).matches(&envelope));
    assert!(!Pattern::map_count(3).matches(&envelope));

    // Matches count range
    assert!(Pattern::map_count_range(1..=3).matches(&envelope));
    assert!(!Pattern::map_count_range(5..=10).matches(&envelope));

    // Test with assertions
    let envelope = envelope.add_assertion("type", "dictionary");
    assert!(Pattern::any_map().matches(&envelope));
    assert!(Pattern::map_count(2).matches(&envelope));

    // The matched paths include the assertion
    let paths = Pattern::any_map().paths(&envelope);
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0], vec![envelope.clone()]);

    // Test with empty map
    let empty_map = Map::new();
    let empty_envelope = Envelope::new(empty_map);
    assert!(Pattern::any_map().matches(&empty_envelope));
    assert!(Pattern::map_count(0).matches(&empty_envelope));
    assert!(!Pattern::map_count(1).matches(&empty_envelope));

    // Test sequence patterns
    let paths = Pattern::sequence(vec![Pattern::any_map(), Pattern::subject()])
        .paths(&envelope);
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].len(), 2);
    assert_eq!(paths[0][0], envelope);
    assert_eq!(paths[0][1], envelope.subject());
}
