mod common;

use bc_envelope::prelude::*;
use indoc::indoc;

// Format each path element on its own line, each line successively indented by
// 4 spaces.
fn format_path(path: &Path) -> String {
    let mut lines = Vec::new();
    for (i, element) in path.iter().enumerate() {
        let indent = " ".repeat(i * 4);
        lines.push(format!(
            "{}{} {}",
            indent,
            element.short_id(DigestDisplayFormat::Short),
            element.format_flat()
        ));
    }
    lines.join("\n")
}

fn format_paths(paths: &Vec<Path>) -> String {
    paths
        .into_iter()
        .map(format_path)
        .collect::<Vec<_>>()
        .join("\n")
}

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

#[test]
fn test_and_pattern() {
    let envelope = Envelope::new(42).add_assertion("an", "assertion");

    // The subject matches the first pattern but not the second.
    let pattern = Pattern::and(vec![Pattern::number(42), Pattern::text("foo")]);
    assert!(!pattern.matches(&envelope));

    // The subject matches both patterns.
    let pattern = Pattern::and(vec![
        Pattern::number_greater_than(40),
        Pattern::number_less_than(50),
    ]);
    assert!(pattern.matches(&envelope));

    // The path includes the assertion.
    let paths = pattern.paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        6cb2ea4a 42 [ "an": "assertion" ]
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    let sequence_pattern =
        Pattern::sequence(vec![pattern.clone(), Pattern::subject()]);
    let paths = sequence_pattern.paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        6cb2ea4a 42 [ "an": "assertion" ]
            7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn test_or_pattern() {
    let envelope = Envelope::new(42).add_assertion("an", "assertion");

    // The subject doesn't match either pattern.
    let pattern = Pattern::or(vec![Pattern::text("bar"), Pattern::text("baz")]);
    assert!(!pattern.matches(&envelope));

    // The subject doesn't match the first pattern but matches the second.
    let pattern = Pattern::or(vec![
        Pattern::text("foo"),
        Pattern::number_greater_than(40),
    ]);
    assert!(pattern.matches(&envelope));

    // The path includes the assertion.
    let paths = pattern.paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        6cb2ea4a 42 [ "an": "assertion" ]
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    let sequence_pattern =
        Pattern::sequence(vec![pattern.clone(), Pattern::subject()]);
    let paths = sequence_pattern.paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        6cb2ea4a 42 [ "an": "assertion" ]
            7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn test_wrapped_pattern() {
    // Does not match non-wrapped subjects.
    let envelope = Envelope::new(42);
    assert!(!Pattern::any_wrapped().matches(&envelope));

    // Matches a wrapped envelope with any subject.
    let envelope = envelope.wrap_envelope();
    assert!(Pattern::any_wrapped().matches(&envelope));

    // The matched paths include the assertion.
    let envelope_with_assertion = envelope.add_assertion("an", "assertion");
    let paths = Pattern::any_wrapped().paths(&envelope_with_assertion);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        169aba00 { 42 } [ "an": "assertion" ]
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // A sequence of one pattern gives the same result as the single pattern.
    let paths = Pattern::sequence(vec![Pattern::any_wrapped()])
        .paths(&envelope_with_assertion);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        169aba00 { 42 } [ "an": "assertion" ]
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // An empty sequence never matches.
    let paths = Pattern::sequence(vec![]).paths(&envelope_with_assertion);
    assert!(paths.is_empty());

    // Matching a wrapped envelope and the subject in a sequence returns a path
    // where the first element is the original wrapped envelope including
    // assertions, and the second element is the still-wrapped subject.
    let paths =
        Pattern::sequence(vec![Pattern::any_wrapped(), Pattern::subject()])
            .paths(&envelope_with_assertion);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        169aba00 { 42 } [ "an": "assertion" ]
            58b1ac6a { 42 }
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // Unwrapping the envelope matches the original subject, and returns a path
    // where the first element is the original wrapped envelope and the second
    // element is the unwrapped subject.
    let paths = Pattern::unwrap().paths(&envelope_with_assertion);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // Matching a wrapped envelope, the subject, and unwrapping it in a sequence
    // returns a path where the first element is the original wrapped envelope
    // including assertions, the second element is the still-wrapped subject,
    // and the third element is the unwrapped subject.
    let paths = Pattern::sequence(vec![
        Pattern::any_wrapped(),
        Pattern::subject(),
        Pattern::unwrap(),
    ])
    .paths(&envelope_with_assertion);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        169aba00 { 42 } [ "an": "assertion" ]
            58b1ac6a { 42 }
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
fn test_search_pattern() {
    // Test searching for text in a simple envelope
    let envelope = Envelope::new("Alice")
        .add_assertion("knows", "Bob")
        .add_assertion("age", 30);

    // Search for any text should find "Alice" (root subject), "Alice" (subject
    // path), "knows", and "Bob"
    let paths = Pattern::search(Pattern::any_text()).paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        a47bb3d4 "Alice" [ "age": 30, "knows": "Bob" ]
        a47bb3d4 "Alice" [ "age": 30, "knows": "Bob" ]
            13941b48 "Alice"
        a47bb3d4 "Alice" [ "age": 30, "knows": "Bob" ]
            0eb5609b "age": 30
                5943be12 "age"
        a47bb3d4 "Alice" [ "age": 30, "knows": "Bob" ]
            78d666eb "knows": "Bob"
                db7dd21c "knows"
        a47bb3d4 "Alice" [ "age": 30, "knows": "Bob" ]
            78d666eb "knows": "Bob"
                13b74194 "Bob"
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // Search for specific text should only find matching elements
    let paths = Pattern::search(Pattern::text("Bob")).paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        a47bb3d4 "Alice" [ "age": 30, "knows": "Bob" ]
            78d666eb "knows": "Bob"
                13b74194 "Bob"
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // Search for numbers should find the age
    let paths = Pattern::search(Pattern::any_number()).paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        a47bb3d4 "Alice" [ "age": 30, "knows": "Bob" ]
            0eb5609b "age": 30
                cf972730 30
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // Search specifically for assertions with objects that are numbers
    let paths =
        Pattern::search(Pattern::assertion_with_object(Pattern::any_number()))
            .paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        a47bb3d4 "Alice" [ "age": 30, "knows": "Bob" ]
            0eb5609b "age": 30
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn test_search_pattern_nested() {
    // Test searching in a more complex nested envelope
    let inner_envelope =
        Envelope::new("Carol").add_assertion("title", "Engineer");

    let envelope = Envelope::new("Alice")
        .add_assertion("knows", inner_envelope)
        .add_assertion("department", "Engineering");

    // Search for all text should find text at all levels
    let paths = Pattern::search(Pattern::any_text()).paths(&envelope);

    assert_eq!(paths.len(), 9);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        a69103e9 "Alice" [ "department": "Engineering", "knows": "Carol" [ "title": "Engineer" ] ]
        a69103e9 "Alice" [ "department": "Engineering", "knows": "Carol" [ "title": "Engineer" ] ]
            13941b48 "Alice"
        a69103e9 "Alice" [ "department": "Engineering", "knows": "Carol" [ "title": "Engineer" ] ]
            2a26d42a "department": "Engineering"
                8aaec3ab "department"
        a69103e9 "Alice" [ "department": "Engineering", "knows": "Carol" [ "title": "Engineer" ] ]
            2a26d42a "department": "Engineering"
                71d7c10e "Engineering"
        a69103e9 "Alice" [ "department": "Engineering", "knows": "Carol" [ "title": "Engineer" ] ]
            c0c35c79 "knows": "Carol" [ "title": "Engineer" ]
                db7dd21c "knows"
        a69103e9 "Alice" [ "department": "Engineering", "knows": "Carol" [ "title": "Engineer" ] ]
            c0c35c79 "knows": "Carol" [ "title": "Engineer" ]
                59e8c540 "Carol" [ "title": "Engineer" ]
        a69103e9 "Alice" [ "department": "Engineering", "knows": "Carol" [ "title": "Engineer" ] ]
            c0c35c79 "knows": "Carol" [ "title": "Engineer" ]
                59e8c540 "Carol" [ "title": "Engineer" ]
                    afb8122e "Carol"
        a69103e9 "Alice" [ "department": "Engineering", "knows": "Carol" [ "title": "Engineer" ] ]
            c0c35c79 "knows": "Carol" [ "title": "Engineer" ]
                59e8c540 "Carol" [ "title": "Engineer" ]
                    a4d32c8f "title": "Engineer"
                        d380cf3f "title"
        a69103e9 "Alice" [ "department": "Engineering", "knows": "Carol" [ "title": "Engineer" ] ]
            c0c35c79 "knows": "Carol" [ "title": "Engineer" ]
                59e8c540 "Carol" [ "title": "Engineer" ]
                    a4d32c8f "title": "Engineer"
                        df9ac43f "Engineer"
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // Verify we can find "Carol" nested inside
    let carol_paths: Vec<_> = paths
        .iter()
        .filter(|path| path.last().unwrap().format_flat().contains("Carol"))
        .collect();
    assert_eq!(carol_paths.len(), 3); // Root envelope, Carol envelope, and Carol subject

    // The path to "Carol" subject should be: envelope -> knows assertion ->
    // Carol envelope -> "Carol"
    let carol_subject_path = carol_paths
        .iter()
        .find(|path| path.last().unwrap().format_flat() == "\"Carol\"")
        .unwrap();
    assert_eq!(carol_subject_path.len(), 4);
}

#[test]
fn test_search_pattern_with_wrapped() {
    // Test searching in wrapped envelopes
    let inner =
        Envelope::new("secret").add_assertion("classification", "top-secret");

    let envelope =
        Envelope::new("Alice").add_assertion("data", inner.wrap_envelope());

    // Search for text should find text in wrapped envelopes too
    let paths = Pattern::search(Pattern::text("secret")).paths(&envelope);
    // println!("{}", format_paths(&paths));
    let expected = indoc! {r#"
        1435493d "Alice" [ "data": { "secret" [ "classification": "top-secret" ] } ]
            a5d4710e "data": { "secret" [ "classification": "top-secret" ] }
                41dca0cd { "secret" [ "classification": "top-secret" ] }
                    f66baec9 "secret" [ "classification": "top-secret" ]
        1435493d "Alice" [ "data": { "secret" [ "classification": "top-secret" ] } ]
            a5d4710e "data": { "secret" [ "classification": "top-secret" ] }
                41dca0cd { "secret" [ "classification": "top-secret" ] }
                    f66baec9 "secret" [ "classification": "top-secret" ]
                        fa445f41 "secret"
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // Should find "secret" twice: once as the wrapped envelope match, once as
    // the subject
    assert_eq!(paths.len(), 2);

    // Both should contain "secret" in the last element
    for path in &paths {
        assert!(path.last().unwrap().format_flat().contains("secret"));
    }
}

#[cfg(feature = "signature")]
#[test]
fn test_search_pattern_credential() {
    use crate::common::test_data::credential;

    let cred = credential();

    // Search for all text in the credential
    let text_paths = Pattern::search(Pattern::any_text()).paths(&cred);
    // Get the last element of each path as a single-element path for output
    let found_elements: Vec<Path> = text_paths
        .iter()
        .map(|path| vec![(*path.last().unwrap()).clone()])
        .collect();
    // println!("{}", format_paths(&found_elements));
    let expected = indoc! {r#"
        9e3bff3a "certificateNumber"
        21c21808 "123-456-789"
        6e5d379f "expirationDate"
        5f82a16a "lastName"
        fe4d5230 "Maxwell"
        222afe69 "issueDate"
        051beee6 "Certificate of Completion"
        3976ef74 "photo"
        231b8527 "This is James Maxwell's photo."
        f13aa855 "professionalDevelopmentHours"
        4395643b "firstName"
        d6d0b768 "James"
        e6bf4dd3 "topics"
        2b191589 "continuingEducationUnits"
        f8489ac1 "Example Electrical Engineering Board"
        8e4e62eb "subject"
        202c10ef "RF and Microwave Engineering"
        f8489ac1 "Example Electrical Engineering Board"
        f106bad1 "Signed by Example Electrical Engineering Board"
    "#}
    .trim();
    assert_actual_expected!(format_paths(&found_elements), expected);

    // Search for specific strings that should be in the credential
    let james_paths = Pattern::search(Pattern::text("James")).paths(&cred);
    assert_eq!(james_paths.len(), 1);

    let maxwell_paths = Pattern::search(Pattern::text("Maxwell")).paths(&cred);
    assert_eq!(maxwell_paths.len(), 1);

    // Search for numbers (should find education units and hours)
    let number_paths =
        Pattern::search(Pattern::assertion_with_object(Pattern::any_number()))
            .paths(&cred);
    // Get the last element of each path as a single-element path for output
    let number_paths: Vec<Path> = number_paths
        .iter()
        .map(|path| vec![(*path.last().unwrap()).clone()])
        .collect();
    let expected = indoc! {r#"
        54b3e1e7 "professionalDevelopmentHours": 15
        8ec5e912 "continuingEducationUnits": 1
    "#}
    .trim();
    assert_actual_expected!(format_paths(&number_paths), expected);
}

#[test]
fn test_digest_pattern() {
    let envelope = Envelope::new("Hello, World!");
    let digest = envelope.digest().into_owned();
    let hex_digest = hex::encode(digest.as_bytes());
    let prefix = &hex_digest[0..8];

    // Test exact digest matching
    assert!(Pattern::digest(digest.clone()).matches(&envelope));
    assert!(
        !Pattern::digest(Digest::from_data([0; 32]).into()).matches(&envelope)
    );

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
    let paths =
        Pattern::digest(Digest::from_data([0; 32]).into()).paths(&envelope);
    assert!(paths.is_empty());
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
        "{} \"Alice\" [ \"knows\": \"Bob\" ]",
        single_assertion_envelope.short_id(DigestDisplayFormat::Short)
    );
    assert_actual_expected!(format_paths(&paths), expected);

    // No match should return empty paths
    let paths = Pattern::any_node().paths(&leaf_envelope);
    assert!(paths.is_empty());
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

#[test]
fn test_mixed_patterns_with_search() {
    // Create a complex envelope structure
    let envelope = Envelope::new("Alice")
        .add_assertion("knows", "Bob")
        .add_assertion("age", 30)
        .add_assertion("secret", Envelope::new("top secret").elide());

    // Search for any node patterns
    let node_paths = Pattern::search(Pattern::any_node()).paths(&envelope);
    assert_eq!(node_paths.len(), 1);

    // Search for any obscured patterns
    let obscured_paths = Pattern::search(Pattern::obscured()).paths(&envelope);
    assert_eq!(obscured_paths.len(), 1);

    // The obscured element should be the elided "top secret"
    let obscured_element = obscured_paths[0].last().unwrap();
    assert!(obscured_element.is_elided());

    // Search for specific digest prefix
    let alice_subject = envelope.subject();
    let alice_digest = alice_subject.digest();
    let alice_hex = hex::encode(alice_digest.as_bytes());
    let alice_prefix = &alice_hex[0..8];

    let digest_paths =
        Pattern::search(Pattern::digest_hex_prefix(alice_prefix))
            .paths(&envelope);
    assert_eq!(digest_paths.len(), 1);

    // The found element should be Alice
    let alice_element = digest_paths[0].last().unwrap();
    assert_eq!(alice_element.extract_subject::<String>().unwrap(), "Alice");
}

#[test]
fn test_node_pattern_with_sequence() {
    // Create envelope with exactly 2 assertions
    let envelope = Envelope::new("Person")
        .add_assertion("name", "Alice")
        .add_assertion("age", 25);

    // Test sequence matching node with 2 assertions then extracting subject
    let paths = Pattern::sequence(vec![
        Pattern::node_with_assertions_count(2),
        Pattern::subject(),
    ])
    .paths(&envelope);

    #[rustfmt::skip]
    let expected = format!(
        "{} \"Person\" [ \"age\": 25, \"name\": \"Alice\" ]\n    {} \"Person\"",
        envelope.short_id(DigestDisplayFormat::Short),
        envelope.subject().short_id(DigestDisplayFormat::Short)
    );
    assert_actual_expected!(format_paths(&paths), expected);
}
