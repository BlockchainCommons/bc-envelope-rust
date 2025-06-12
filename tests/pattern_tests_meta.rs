mod common;

use bc_envelope::prelude::*;
use indoc::indoc;

use crate::common::pattern_utils::*;

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
