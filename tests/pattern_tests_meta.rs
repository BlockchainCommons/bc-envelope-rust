mod common;

use bc_envelope::prelude::*;
use indoc::indoc;

use crate::common::pattern_utils::*;

#[test]
fn test_empty_sequence_pattern() {
    let envelope = Envelope::new(42);

    // An empty sequence pattern never matches.
    let pattern = Pattern::sequence(vec![]);
    let paths = pattern.paths(&envelope);
    assert!(paths.is_empty());
}

#[test]
fn test_one_element_sequence_pattern() {
    let envelope = Envelope::new(42);

    let number_pattern = Pattern::number(42);
    let expected  = indoc! {r#"
        7f83f7bd 42
    "#}.trim();
    let paths = number_pattern.paths(&envelope);
    assert_actual_expected!(format_paths(&paths), expected);

    // A sequence of one pattern gives the same result as the single pattern.
    let pattern = Pattern::sequence(vec![number_pattern]);
    let paths = pattern.paths(&envelope);
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
fn optional_single_or_pattern() {
    let inner = Envelope::new("data");
    let wrapped = inner.clone().wrap_envelope();

    println!("=== Tree Format ===");
    println!("inner tree:\n\n{}\n", inner.tree_format());
    println!("wrapped tree:\n\n{}\n", wrapped.tree_format());

    // let pat = Pattern::sequence(vec![
    //     Pattern::repeat_greedy(Pattern::wrapped(), 0..=1),
    //     Pattern::subject(),
    // ]);

    let pat = Pattern::subject();

    // let pat = Pattern::or(vec![
    //     // Pattern::sequence(vec![Pattern::wrapped(), Pattern::unwrap()]),
    //     Pattern::subject(),
    // ]);

    // let pat = Pattern::repeat_greedy(Pattern::wrapped(), 0..=1);

    assert!(pat.matches(&inner));
    assert!(pat.matches(&wrapped));

    let inner_paths = pat.paths(&inner);
    let wrapped_paths = pat.paths(&wrapped);

    println!("=== Matching Paths ===");
    println!("inner matches {} paths:\n\n{}\n", inner_paths.len(), format_paths(&inner_paths));
    println!("wrapped matches {} paths:\n\n{}\n", wrapped_paths.len(), format_paths(&wrapped_paths));

    // // shortest path when unwrapped
    // assert_eq!(pat.paths(&inner).len(), 1);
    // // wrapped path has two elements
    // assert_eq!(pat.paths(&wrapped).len(), 2);
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

#[test]
fn test_not_pattern() {
    // Create a test envelope
    let envelope = Envelope::new("test_subject")
        .add_assertion("key1", "value1")
        .add_assertion("key2", "value2")
        .add_assertion("number", 42);

    // Test not pattern with text pattern that doesn't match
    let not_matches = Pattern::not_matching(Pattern::text("non_matching_text"))
        .matches(&envelope);
    assert!(
        not_matches,
        "Should match when the inner pattern doesn't match"
    );

    // Test not pattern with text pattern that does match
    let not_matches =
        Pattern::not_matching(Pattern::text("test_subject")).matches(&envelope);
    assert!(
        !not_matches,
        "Should not match when the inner pattern matches"
    );

    // Test not pattern with object pattern - this should find no matches
    // because we have an assertion with object 42
    let not_patterns = Pattern::search(Pattern::not_matching(Pattern::object(
        Pattern::number(42),
    )))
    .paths(&envelope);

    // Should not match the assertion with object 42, but will match other
    // elements
    let found_objects = not_patterns
        .iter()
        .filter(|path| path.last().unwrap().is_assertion())
        .filter_map(|path| {
            let assertion = path.last().unwrap();
            if let Ok(obj) = assertion.extract_object::<i32>() {
                Some(obj)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    assert!(
        !found_objects.contains(&42),
        "Should not match assertions with object 42"
    );

    // Test combination of not pattern with other patterns
    let complex_pattern = Pattern::and(vec![
        Pattern::not_matching(Pattern::text("wrong_subject")),
        Pattern::assertion_with_predicate(Pattern::text("key1")),
    ]);

    let matches = complex_pattern.matches(&envelope);
    assert!(matches, "Complex pattern with not should match");

    // The path includes the assertion for a successful not pattern
    let pattern = Pattern::not_matching(Pattern::text("wrong"));
    let paths = pattern.paths(&envelope);

    // Instead of checking exact digest (which can change), check the content
    assert_eq!(paths.len(), 1, "Should have one path");
    let returned_envelope = paths[0][0].clone();
    assert_eq!(
        returned_envelope.extract_subject::<String>().unwrap(),
        "test_subject"
    );
}

#[test]
fn test_not_pattern_with_search() {
    // Create a nested envelope structure
    let inner_envelope =
        Envelope::new("inner").add_assertion("inner_key", "inner_value");

    let outer_envelope =
        Envelope::new("outer").add_assertion("contains", inner_envelope);

    // Search for elements that are NOT obscured (everything in this case)
    let not_obscured_paths =
        Pattern::search(Pattern::not_matching(Pattern::obscured()))
            .paths(&outer_envelope);

    // We should find multiple matches (everything, since nothing is obscured)
    assert!(
        !not_obscured_paths.is_empty(),
        "Should find elements that are not obscured"
    );

    // Create envelope with elided content
    let envelope_with_elided = Envelope::new("test")
        .add_assertion("visible", "data")
        .add_assertion("hidden", Envelope::new("secret").elide());

    // Search for elements that are NOT elided
    let not_elided_paths =
        Pattern::search(Pattern::not_matching(Pattern::obscured()))
            .paths(&envelope_with_elided);

    // Should find multiple elements that are not elided
    assert!(
        !not_elided_paths.is_empty(),
        "Should find elements that are not elided"
    );

    // Verify we didn't match the elided element
    for path in &not_elided_paths {
        if let Some(element) = path.last() {
            assert!(!element.is_elided(), "Should not match elided elements");
        }
    }
}
