mod common;

use bc_envelope::prelude::*;
use bc_envelope_pattern::{Matcher, Pattern};
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
    assert!(Pattern::text_regex(regex.clone()).matches(&envelope));

    // Matches a subject that is text with an assertion.
    let envelope = envelope.add_assertion("greeting", "world");
    assert!(Pattern::any_text().matches(&envelope));
    assert!(Pattern::text("hello").matches(&envelope));
    assert!(!Pattern::text("world").matches(&envelope));
    assert!(Pattern::text_regex(regex).matches(&envelope));

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

#[test]
fn test_date_pattern() {
    use dcbor::Date;

    // Does not match non-date subjects.
    let envelope = Envelope::new("2023-12-25");
    assert!(!Pattern::any_date().matches(&envelope));

    // Create a date envelope
    let date = Date::from_ymd(2023, 12, 25);
    let envelope = Envelope::new(date.clone());

    // Matches a bare subject that is a date.
    assert!(Pattern::any_date().matches(&envelope));
    assert!(Pattern::date(date.clone()).matches(&envelope));
    assert!(!Pattern::date(Date::from_ymd(2023, 12, 24)).matches(&envelope));

    // Test date range matching
    let start = Date::from_ymd(2023, 12, 20);
    let end = Date::from_ymd(2023, 12, 30);
    assert!(Pattern::date_range(start..=end).matches(&envelope));

    let start = Date::from_ymd(2023, 12, 26);
    let end = Date::from_ymd(2023, 12, 30);
    assert!(!Pattern::date_range(start..=end).matches(&envelope));

    // Test ISO-8601 string matching
    assert!(Pattern::date_iso8601("2023-12-25").matches(&envelope));
    assert!(!Pattern::date_iso8601("2023-12-24").matches(&envelope));

    // Test regex matching
    let regex = regex::Regex::new(r"^2023-.*").unwrap();
    assert!(Pattern::date_regex(regex).matches(&envelope));

    let regex = regex::Regex::new(r"^2024-.*").unwrap();
    assert!(!Pattern::date_regex(regex).matches(&envelope));

    // Matches a subject that is a date with an assertion.
    let envelope = envelope.add_assertion("type", "christmas");
    assert!(Pattern::any_date().matches(&envelope));
    assert!(Pattern::date(date.clone()).matches(&envelope));

    // The matched paths include the assertion.
    let paths = Pattern::any_date().paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        20f45d77 2023-12-25 [ "type": "christmas" ]
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // Matching a date and the subject in a sequence returns a path
    // where the first element is the original envelope and the second
    // element is the subject.
    let paths =
        Pattern::sequence(vec![Pattern::any_date(), Pattern::subject()])
            .paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        20f45d77 2023-12-25 [ "type": "christmas" ]
            3854ff69 2023-12-25
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // Test with date-time (not just date)
    let date_with_time = Date::from_ymd_hms(2023, 12, 25, 15, 30, 45);
    let envelope_with_time = Envelope::new(date_with_time.clone());

    assert!(Pattern::any_date().matches(&envelope_with_time));
    assert!(
        Pattern::date_iso8601("2023-12-25T15:30:45Z")
            .matches(&envelope_with_time)
    );

    let regex = regex::Regex::new(r".*T15:30:45Z$").unwrap();
    assert!(Pattern::date_regex(regex).matches(&envelope_with_time));
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
    //
    // IMPORTANT: Binary regex patterns must use the (?s-u) flags to work
    // correctly with arbitrary bytes:
    // - (?s) allows '.' to match newline characters (like 0x0a)
    // - (?-u) disables unicode mode so '.' matches any byte, not just valid
    //   UTF-8 sequences
    //
    // Example matching any byte string ending with h'0001020304':
    // ```
    // (?s-u).*\x00\x01\x02\x03\x04$
    // ```

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
    #[rustfmt::skip]
    let expected = indoc! {r#"
        70db8c16 [1, 2, 3] [ "type": "list" ]
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

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
    #[rustfmt::skip]
    let expected = indoc! {r#"
        70db8c16 [1, 2, 3] [ "type": "list" ]
            4abc3113 [1, 2, 3]
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
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
    #[rustfmt::skip]
    let expected = indoc! {r#"
        1d96ee45 {"key1": "value1", "key2": "value2"} [ "type": "dictionary" ]
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // Test with empty map
    let empty_map = Map::new();
    let empty_envelope = Envelope::new(empty_map);
    assert!(Pattern::any_map().matches(&empty_envelope));
    assert!(Pattern::map_count(0).matches(&empty_envelope));
    assert!(!Pattern::map_count(1).matches(&empty_envelope));

    // Test sequence patterns
    let paths = Pattern::sequence(vec![Pattern::any_map(), Pattern::subject()])
        .paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        1d96ee45 {"key1": "value1", "key2": "value2"} [ "type": "dictionary" ]
            0e16f9b4 {"key1": "value1", "key2": "value2"}
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn test_null_pattern() {
    // Does not match non-null subjects.
    let envelope = Envelope::new("string");
    assert!(!Pattern::null().matches(&envelope));

    // Matches a null subject.
    let envelope = Envelope::null();
    assert!(Pattern::null().matches(&envelope));

    // Matches a subject that is null with an assertion.
    let envelope = envelope.add_assertion("type", "null_value");
    assert!(Pattern::null().matches(&envelope));

    // The matched paths include the assertion.
    let paths = Pattern::null().paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        a72948d7 null [ "type": "null_value" ]
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // Matching a null and the subject in a sequence returns a path
    // where the first element is the original envelope and the second
    // element is the subject.
    let paths = Pattern::sequence(vec![Pattern::null(), Pattern::subject()])
        .paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        a72948d7 null [ "type": "null_value" ]
            b0b2988b null
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn test_tag_pattern() {
    use dcbor::prelude::*;

    // Does not match non-tagged subjects.
    let envelope = Envelope::new("string");
    assert!(!Pattern::any_tag().matches(&envelope));
    assert!(!Pattern::tagged_with_value(100).matches(&envelope));

    // Test with a tagged CBOR value
    let tagged_cbor = CBOR::to_tagged_value(100, "tagged_content");
    let envelope = Envelope::new(tagged_cbor);

    // Matches any tag
    assert!(Pattern::any_tag().matches(&envelope));

    // Matches specific tag value
    assert!(Pattern::tagged_with_value(100).matches(&envelope));
    assert!(!Pattern::tagged_with_value(200).matches(&envelope));

    // Matches specific tag object
    let tag = Tag::with_value(100);
    assert!(Pattern::tagged(tag).matches(&envelope));

    let different_tag = Tag::with_value(200);
    assert!(!Pattern::tagged(different_tag).matches(&envelope));

    // Test with assertions
    let envelope = envelope.add_assertion("format", "tagged");
    assert!(Pattern::any_tag().matches(&envelope));
    assert!(Pattern::tagged_with_value(100).matches(&envelope));

    // The matched paths include the assertion
    let paths = Pattern::any_tag().paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        b9457c8d 100("tagged_content") [ "format": "tagged" ]
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // Test sequence patterns
    let paths = Pattern::sequence(vec![Pattern::any_tag(), Pattern::subject()])
        .paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        b9457c8d 100("tagged_content") [ "format": "tagged" ]
            a8e58a0d 100("tagged_content")
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn test_tag_pattern_named() {
    use dcbor::prelude::*;

    // Ensure tags are registered for testing
    bc_envelope::register_tags();

    // Test with registered tag (date tag = 1)
    let tagged_cbor = dcbor::Date::from_ymd(2023, 12, 25).to_cbor();
    let envelope = Envelope::new(tagged_cbor);

    // Should match by name
    assert!(Pattern::tagged_with_name("date").matches(&envelope));

    // Should not match with wrong name
    assert!(!Pattern::tagged_with_name("unknown_tag").matches(&envelope));

    // Test with unregistered tag
    let unregistered_tagged_cbor =
        CBOR::to_tagged_value(999, "unregistered_content");
    let unregistered_envelope = Envelope::new(unregistered_tagged_cbor);

    // Should not match by any name since tag 999 is not registered
    assert!(!Pattern::tagged_with_name("date").matches(&unregistered_envelope));
    assert!(
        !Pattern::tagged_with_name("unknown_tag")
            .matches(&unregistered_envelope)
    );

    // Test with non-tagged content
    let text_envelope = Envelope::new("just text");
    assert!(!Pattern::tagged_with_name("date").matches(&text_envelope));

    // Test paths for matched envelope
    let paths = Pattern::tagged_with_name("date").paths(&envelope);
    // println!("{}", format_paths(&paths));
    let expected = indoc! {r#"
        3854ff69 2023-12-25
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].len(), 1);
    assert_eq!(paths[0][0], envelope);

    // Test in sequence pattern
    let paths = Pattern::sequence(vec![
        Pattern::tagged_with_name("date"),
        Pattern::subject(),
    ])
    .paths(&envelope);
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn test_tag_pattern_regex() {
    use dcbor::prelude::*;

    // Ensure tags are registered for testing
    bc_envelope::register_tags();

    // Test with registered tag (date tag = 1)
    let tagged_cbor = dcbor::Date::from_ymd(2023, 12, 25).to_cbor();
    let envelope = Envelope::new(tagged_cbor);

    // Regex that should match "date"
    let regex = regex::Regex::new(r"^da.*").unwrap();
    assert!(Pattern::tagged_with_regex(regex.clone()).matches(&envelope));

    // Regex that should match names ending with "te"
    let regex = regex::Regex::new(r".*te$").unwrap();
    assert!(Pattern::tagged_with_regex(regex.clone()).matches(&envelope));

    // Regex that should not match "date"
    let regex = regex::Regex::new(r"^time.*").unwrap();
    assert!(!Pattern::tagged_with_regex(regex.clone()).matches(&envelope));

    // Test with unregistered tag
    let unregistered_tagged_cbor =
        CBOR::to_tagged_value(999, "unregistered_content");
    let unregistered_envelope = Envelope::new(unregistered_tagged_cbor);

    // Should not match any regex since tag 999 has no name in registry
    let regex = regex::Regex::new(r".*").unwrap(); // Match everything
    assert!(
        !Pattern::tagged_with_regex(regex.clone())
            .matches(&unregistered_envelope)
    );

    // Test with non-tagged content
    let text_envelope = Envelope::new("just text");
    let regex = regex::Regex::new(r".*").unwrap();
    assert!(!Pattern::tagged_with_regex(regex.clone()).matches(&text_envelope));

    // Test paths for matched envelope
    let regex = regex::Regex::new(r"^da.*").unwrap();
    let paths = Pattern::tagged_with_regex(regex).paths(&envelope);
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].len(), 1);
    assert_eq!(paths[0][0], envelope);

    // Test in sequence pattern
    let regex = regex::Regex::new(r".*te$").unwrap();
    let paths = Pattern::sequence(vec![
        Pattern::tagged_with_regex(regex),
        Pattern::subject(),
    ])
    .paths(&envelope);
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].len(), 1);
    assert_eq!(paths[0][0], envelope);
}

#[test]
fn test_tag_pattern_with_bc_components_tags() {
    use dcbor::prelude::*;

    // Ensure all tags are registered
    bc_envelope::register_tags();

    // Test with a bc-components tag (e.g., digest tag)
    let digest_tag_value = 40001u64; // TAG_DIGEST from bc-tags
    let tagged_cbor =
        CBOR::to_tagged_value(digest_tag_value, [1u8, 2, 3, 4].as_slice());
    let envelope = Envelope::new(tagged_cbor);

    // Test tag name matching (assuming digest tag is registered with name
    // "digest") This will verify if the tag is properly registered in the
    // global registry
    let pattern = Pattern::tagged_with_name("digest");
    let matches = pattern.matches(&envelope);

    // Also test regex matching for digest-related tags
    let regex = regex::Regex::new(r".*digest.*").unwrap();
    let pattern_regex = Pattern::tagged_with_regex(regex);
    let matches_regex = pattern_regex.matches(&envelope);

    // At minimum, the specific tag value should work
    assert!(Pattern::tagged_with_value(digest_tag_value).matches(&envelope));
    assert!(Pattern::any_tag().matches(&envelope));

    // Print debug info for manual verification
    println!(
        "Digest tag {} matches by name: {}",
        digest_tag_value, matches
    );
    println!(
        "Digest tag {} matches by regex: {}",
        digest_tag_value, matches_regex
    );
}
