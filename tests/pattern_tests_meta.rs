mod common;

use bc_envelope::{pattern::Greediness, prelude::*};
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
fn test_and_pattern() {
    let envelope = Envelope::new(42).add_assertion("an", "assertion");

    // A pattern that requires the envelope to match both a number and a text,
    // which is impossible.
    let impossible_pattern =
        Pattern::and(vec![Pattern::number(42), Pattern::text("foo")]);
    assert!(!impossible_pattern.matches(&envelope));

    // A pattern that requires the envelope to match both a number greater
    // than 40 and a number less than 50, which is possible.
    let number_range_pattern = Pattern::and(vec![
        Pattern::number_greater_than(40),
        Pattern::number_less_than(50),
    ]);
    assert!(number_range_pattern.matches(&envelope));

    // The path includes the assertion.
    let paths = number_range_pattern.paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        6cb2ea4a 42 [ "an": "assertion" ]
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // A sequence pattern that includes the number range pattern and then
    // extracts the subject.
    let number_range_with_subject_pattern =
        Pattern::sequence(vec![number_range_pattern, Pattern::subject()]);
    let paths = number_range_with_subject_pattern.paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        6cb2ea4a 42 [ "an": "assertion" ]
            7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn test_or_pattern() {
    // A pattern that requires the envelope to match either the string "foo" or
    // the string "bar".
    let pattern = Pattern::or(vec![Pattern::text("bar"), Pattern::text("baz")]);

    // An envelope that is a number, so it doesn't match the pattern.
    let envelope = Envelope::new(42).add_assertion("an", "assertion");
    assert!(!pattern.matches(&envelope));

    // A pattern that requires the envelope to match either the string "foo" or
    // a number greater than 40.
    let foo_or_greater_than_40_pattern = Pattern::or(vec![
        Pattern::text("foo"),
        Pattern::number_greater_than(40),
    ]);
    // The subject doesn't match the first pattern but matches the second.
    assert!(foo_or_greater_than_40_pattern.matches(&envelope));

    // The match path includes the assertion.
    let paths = foo_or_greater_than_40_pattern.paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        6cb2ea4a 42 [ "an": "assertion" ]
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    let foo_or_greater_than_40_with_subject_pattern = Pattern::sequence(vec![
        foo_or_greater_than_40_pattern,
        Pattern::subject(),
    ]);
    let paths = foo_or_greater_than_40_with_subject_pattern.paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        6cb2ea4a 42 [ "an": "assertion" ]
            7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn test_one_element_sequence_pattern() {
    // A pattern that matches a the number 42.
    let number_pattern = Pattern::number(42);

    let envelope = Envelope::new(42);
    let expected = indoc! {r#"
        7f83f7bd 42
    "#}
    .trim();
    let paths = number_pattern.paths(&envelope);
    assert_actual_expected!(format_paths(&paths), expected);

    // A sequence of one pattern gives the same result as the single pattern.
    let pattern = Pattern::sequence(vec![number_pattern]);
    let paths = pattern.paths(&envelope);
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn test_wrapped_sequence() {
    let env_1 = Envelope::new("data");
    let wrapped_1 = env_1.wrap_envelope();
    let wrapped_2 = wrapped_1.wrap_envelope();
    let wrapped_3 = wrapped_2.wrap_envelope();
    let wrapped_4 = wrapped_3.wrap_envelope();

    // println!("{}", wrapped_4.tree_format());
    let expected = indoc! {r#"
        25cb582c WRAPPED
            c1426a18 subj WRAPPED
                ee8cade0 subj WRAPPED
                    febc1555 subj WRAPPED
                        e909da9a subj "data"
    "#}
    .trim();
    assert_actual_expected!(wrapped_4.tree_format(), expected);

    // println!("{}", wrapped_4.format_flat());
    let expected = indoc! {r#"
        { { { { "data" } } } }
    "#}
    .trim();
    assert_actual_expected!(wrapped_4.format_flat(), expected);

    // A pattern that matches a single wrapped envelope.
    let wrapped_1_pattern = Pattern::sequence(vec![Pattern::wrapped()]);
    let paths = wrapped_1_pattern.paths(&wrapped_4);
    // println!("{}", format_paths(&paths));
    #[rustfmt::skip]
    let expected = indoc! {r#"
        25cb582c { { { { "data" } } } }
            c1426a18 { { { "data" } } }
    "#}
    .trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // A pattern that matches two wrapped envelopes in sequence.
    let wrapped_2_pattern =
        Pattern::sequence(vec![Pattern::wrapped(), Pattern::wrapped()]);
    let paths = wrapped_2_pattern.paths(&wrapped_4);
    // println!("{}", format_paths(&paths));
    #[rustfmt::skip]
    let expected = indoc! {r#"
        25cb582c { { { { "data" } } } }
            c1426a18 { { { "data" } } }
                ee8cade0 { { "data" } }
    "#}
    .trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // A pattern that matches three wrapped envelopes in sequence.
    let wrapped_3_pattern = Pattern::sequence(vec![
        Pattern::wrapped(),
        Pattern::wrapped(),
        Pattern::wrapped(),
    ]);
    let paths = wrapped_3_pattern.paths(&wrapped_4);
    // println!("{}", format_paths(&paths));
    #[rustfmt::skip]
    let expected = indoc! {r#"
        25cb582c { { { { "data" } } } }
            c1426a18 { { { "data" } } }
                ee8cade0 { { "data" } }
                    febc1555 { "data" }
    "#}
    .trim();
    assert_actual_expected!(format_paths(&paths), expected);

    // A pattern that matches four wrapped envelopes in sequence.
    let wrapped_4_pattern = Pattern::sequence(vec![
        Pattern::wrapped(),
        Pattern::wrapped(),
        Pattern::wrapped(),
        Pattern::wrapped(),
    ]);
    let paths = wrapped_4_pattern.paths(&wrapped_4);
    // println!("{}", format_paths(&paths));
    #[rustfmt::skip]
    let expected = indoc! {r#"
        25cb582c { { { { "data" } } } }
            c1426a18 { { { "data" } } }
                ee8cade0 { { "data" } }
                    febc1555 { "data" }
                        e909da9a "data"
    "#}
    .trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn optional_wrapped_pattern() {
    // A pattern that matches an envelope that may or may not be wrapped.
    let optional_wrapped_pattern = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), 0..=1, Greediness::Greedy),
        Pattern::any_number(),
    ]);

    let inner = Envelope::new(42);
    let wrapped = inner.wrap_envelope();

    let inner_paths = optional_wrapped_pattern.paths(&inner);
    #[rustfmt::skip]
    let expected  = indoc! {r#"
        7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&inner_paths), expected);

    let wrapped_paths = optional_wrapped_pattern.paths(&wrapped);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        58b1ac6a { 42 }
            7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&wrapped_paths), expected);
}

#[test]
fn test_search_pattern() {
    // A pattern that searches for any text in the envelope
    let text_search_pattern = Pattern::search(Pattern::any_text());

    // Test searching for text in a simple envelope
    let envelope = Envelope::new("Alice")
        .add_assertion("knows", "Bob")
        .add_assertion("age", 30);

    let text_search_paths = text_search_pattern.paths(&envelope);
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
    assert_actual_expected!(format_paths(&text_search_paths), expected);

    // A pattern that searches for the text "Bob" in the envelope
    let bob_search_pattern = Pattern::search(Pattern::text("Bob"));
    let bob_search_paths = bob_search_pattern.paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        a47bb3d4 "Alice" [ "age": 30, "knows": "Bob" ]
            78d666eb "knows": "Bob"
                13b74194 "Bob"
    "#}.trim();
    assert_actual_expected!(format_paths(&bob_search_paths), expected);

    // A pattern that searches for any number in the envelope
    let number_search_pattern = Pattern::search(Pattern::any_number());
    let number_search_paths = number_search_pattern.paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        a47bb3d4 "Alice" [ "age": 30, "knows": "Bob" ]
            0eb5609b "age": 30
                cf972730 30
    "#}.trim();
    assert_actual_expected!(format_paths(&number_search_paths), expected);

    // A pattern that searches for any assertion with an object
    // that is a number
    let number_object_search_pattern =
        Pattern::search(Pattern::assertion_with_object(Pattern::any_number()));
    let number_object_search_paths =
        number_object_search_pattern.paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        a47bb3d4 "Alice" [ "age": 30, "knows": "Bob" ]
            0eb5609b "age": 30
    "#}.trim();
    assert_actual_expected!(
        format_paths(&number_object_search_paths),
        expected
    );
}

#[test]
#[ignore]
fn test_search_pattern_nested() {
    // A pattern that searches for any text in the envelope
    let text_search_pattern = Pattern::search(Pattern::any_text());

    // Test searching in a more complex nested envelope
    let inner_envelope =
        Envelope::new("Carol").add_assertion("title", "Engineer");

    let envelope = Envelope::new("Alice")
        .add_assertion("knows", inner_envelope)
        .add_assertion("department", "Engineering");

    // Search for all text should find text at all levels
    let text_search_paths = text_search_pattern.paths(&envelope);

    assert_eq!(text_search_paths.len(), 9);
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
    assert_actual_expected!(format_paths(&text_search_paths), expected);

    // Verify we can find "Carol" nested inside
    let carol_paths: Vec<_> = text_search_paths
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
#[ignore]
fn test_search_pattern_with_wrapped() {
    // A pattern that searches for the text "secret" in the envelope
    let secret_text_search_pattern = Pattern::search(Pattern::text("secret"));

    let inner =
        Envelope::new("secret").add_assertion("classification", "top-secret");
    let envelope =
        Envelope::new("Alice").add_assertion("data", inner.wrap_envelope());

    let secret_text_search_paths = secret_text_search_pattern.paths(&envelope);
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
    assert_actual_expected!(format_paths(&secret_text_search_paths), expected);

    // A pattern that searches for any text containing the word "secret"
    let secret_regex_search_pattern = Pattern::search(Pattern::text_regex(
        regex::Regex::new("secret").unwrap(),
    ));
    let secret_regex_search_paths =
        secret_regex_search_pattern.paths(&envelope);
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
        1435493d "Alice" [ "data": { "secret" [ "classification": "top-secret" ] } ]
            a5d4710e "data": { "secret" [ "classification": "top-secret" ] }
                41dca0cd { "secret" [ "classification": "top-secret" ] }
                    f66baec9 "secret" [ "classification": "top-secret" ]
                        7e14bb9e "classification": "top-secret"
                            c2d8f15f "top-secret"
    "#}.trim();
    assert_actual_expected!(format_paths(&secret_regex_search_paths), expected);
}

#[cfg(feature = "signature")]
#[test]
#[ignore]
fn test_search_pattern_credential() {
    use crate::common::test_data::credential;

    // A pattern that searches for any text in the envelepe
    let text_search_pattern = Pattern::search(Pattern::any_text());

    let cred = credential();
    // println!("{}", cred.tree_format());
    let expected = indoc! {r#"
        0b721f78 NODE
            397a2d4c subj WRAPPED
                8122ffa9 subj NODE
                    10d3de01 subj ARID(4676635a)
                    1f9ff098 ASSERTION
                        9e3bff3a pred "certificateNumber"
                        21c21808 obj "123-456-789"
                    36c254d0 ASSERTION
                        6e5d379f pred "expirationDate"
                        639ae9bf obj 2028-01-01
                    3c114201 ASSERTION
                        5f82a16a pred "lastName"
                        fe4d5230 obj "Maxwell"
                    4a9b2e4d ASSERTION
                        222afe69 pred "issueDate"
                        cb67f31d obj 2020-01-01
                    4d67bba0 ASSERTION
                        2be2d79b pred 'isA'
                        051beee6 obj "Certificate of Completion"
                    5171cbaf ASSERTION
                        3976ef74 pred "photo"
                        231b8527 obj "This is James Maxwell's photo."
                    54b3e1e7 ASSERTION
                        f13aa855 pred "professionalDevelopmentHours"
                        dc0e9c36 obj 15
                    5dc6d4e3 ASSERTION
                        4395643b pred "firstName"
                        d6d0b768 obj "James"
                    68895d8e ASSERTION
                        e6bf4dd3 pred "topics"
                        543fcc09 obj ["Subject 1", "Subject 2"]
                    8ec5e912 ASSERTION
                        2b191589 pred "continuingEducationUnits"
                        4bf5122f obj 1
                    9b3d4785 ASSERTION
                        af10ee92 pred 'controller'
                        f8489ac1 obj "Example Electrical Engineering Board"
                    caf5ced3 ASSERTION
                        8e4e62eb pred "subject"
                        202c10ef obj "RF and Microwave Engineering"
                    d3e0cc15 ASSERTION
                        6dd16ba3 pred 'issuer'
                        f8489ac1 obj "Example Electrical Engineering Board"
            46a02aaf ASSERTION
                d0e39e78 pred 'signed'
                34c14941 obj Signature
            e6d7fca0 ASSERTION
                0fcd6a39 pred 'note'
                f106bad1 obj "Signed by Example Electrical Engineering…"
    "#}.trim();
    assert_actual_expected!(cred.tree_format(), expected);

    // Search for all text in the credential
    let text_paths = text_search_pattern.paths(&cred);
    // Get the last element of each path as a single-element path for output
    let found_elements: Vec<Path> = text_paths
        .iter()
        .map(|path| vec![(*path.last().unwrap()).clone()])
        .collect();
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

    // The above pattern is returning some text elements more than once.
    //
    // Current actual output:

    /*
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
     */
}

#[test]
#[ignore]
fn test_search_pattern_credential_2() {
    // // A pattern that searches for the text "James" in the credential
    // let james_search_pattern = Pattern::search(Pattern::text("James"));
    // // Search for specific strings that should be in the credential
    // let james_paths = james_search_pattern.paths(&cred);
    // assert_eq!(james_paths.len(), 1);

    // // A pattern that searches for the text "Maxwell" in the credential
    // let maxwell_search_pattern = Pattern::search(Pattern::text("Maxwell"));
    // let maxwell_paths = maxwell_search_pattern.paths(&cred);
    // assert_eq!(maxwell_paths.len(), 1);

    // // A pattern that searches for numbers in the credential
    // let number_search_pattern =
    //     Pattern::search(Pattern::assertion_with_object(Pattern::any_number()));
    // // Should find education units and hours
    // let number_paths = number_search_pattern.paths(&cred);
    // // Get the last element of each path as a single-element path for output
    // let number_paths: Vec<Path> = number_paths
    //     .iter()
    //     .map(|path| vec![(*path.last().unwrap()).clone()])
    //     .collect();
    // let expected = indoc! {r#"
    //     54b3e1e7 "professionalDevelopmentHours": 15
    //     8ec5e912 "continuingEducationUnits": 1
    // "#}
    // .trim();
    // assert_actual_expected!(format_paths(&number_paths), expected);
}

#[test]
#[ignore]
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
#[ignore]
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
