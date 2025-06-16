mod common;

use bc_envelope::prelude::*;
use bc_envelope_pattern::{Matcher, Pattern};
use indoc::indoc;

use crate::common::{pattern_utils::*, test_data::*};

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

#[test]
fn test_redacted_credential_patterns() {
    let redacted_credential = redacted_credential();
    // println!("{}", redacted_credential.format());
    let expected = indoc! {r#"
        {
            ARID(4676635a) [
                'isA': "Certificate of Completion"
                "expirationDate": 2028-01-01
                "firstName": "James"
                "lastName": "Maxwell"
                "subject": "RF and Microwave Engineering"
                'issuer': "Example Electrical Engineering Board"
                ELIDED (7)
            ]
        } [
            'note': "Signed by Example Electrical Engineering Board"
            'signed': Signature
        ]
    "#}
    .trim();
    assert_actual_expected!(redacted_credential.format(), expected);

    // Search for the obscured paths
    let paths =
        Pattern::search(Pattern::obscured()).paths(&redacted_credential);
    // println!("{}", &format_paths(&paths));
    assert_eq!(paths.len(), 7);

    // Get the last elements of each path to check the obscured elements
    let obscured_elements: Vec<_> = paths
        .iter()
        .map(|path| path.last().unwrap().clone())
        .collect();

    // Print them formatted as paths
    let formatted_obscured: Vec<_> = obscured_elements
        .iter()
        .map(|el| vec![el.clone()])
        .collect();
    // println!("{}", format_paths(&formatted_obscured));
    let expected = indoc! {r#"
        1f9ff098 ELIDED
        4a9b2e4d ELIDED
        5171cbaf ELIDED
        54b3e1e7 ELIDED
        68895d8e ELIDED
        8ec5e912 ELIDED
        9b3d4785 ELIDED
    "#}
    .trim();
    assert_actual_expected!(format_paths(&formatted_obscured), expected);

    // Get the un-redacted credential
    let credential = credential();

    // Get the digest of the first obscured element
    let first_obscured_digest =
        obscured_elements.first().unwrap().digest().into_owned();

    // Find the digest of the first obscured element in the un-redacted
    // credential
    let found_paths =
        Pattern::search(Pattern::digest(first_obscured_digest.clone()))
            .paths(&credential);
    // println!("{}", format_paths(&found_paths));
    assert_eq!(found_paths.len(), 1);

    // Get the last element of the found path
    let found_element = found_paths[0].last().unwrap();
    // println!("{}", found_element.format());
    let expected = indoc! {r#"
        "certificateNumber": "123-456-789"
    "#}
    .trim();
    assert_actual_expected!(found_element.format(), expected);

    // The digest of the found element should match the first obscured element
    assert_eq!(
        found_element.digest().into_owned(),
        first_obscured_digest,
        "The found element's digest should match the first obscured element's digest"
    );
}
