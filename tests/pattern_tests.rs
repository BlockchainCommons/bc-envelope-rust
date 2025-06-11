mod common;
use bc_envelope::prelude::*;
use common::test_data::*;
use indoc::indoc;

// Format each path element on its own line, each line successively indented by
// 4 spaces.
fn format_path(path: &Path) -> String {
    let mut lines = Vec::new();
    for (i, element) in path.iter().enumerate() {
        let indent = " ".repeat(i * 4);
        lines.push(format!("{}{}", indent, element.format_flat()));
    }
    lines.join("\n")
}

fn format_paths(paths: &Vec<Path>) -> String {
    paths
        .into_iter()
        .map(format_path)
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn print_paths(paths: &Vec<Path>) {
    println!("{}", format_paths(paths));
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
        true [ "an": "assertion" ]
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
        true [ "an": "assertion" ]
            true
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
        42 [ "an": "assertion" ]
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
        42 [ "an": "assertion" ]
            42
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
        "hello" [ "greeting": "world" ]
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
        "hello" [ "greeting": "world" ]
            "hello"
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
        42 [ "an": "assertion" ]
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    let sequence_pattern = Pattern::sequence(vec![
        pattern.clone(),
        Pattern::subject(),
    ]);
    let paths = sequence_pattern.paths(&envelope);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        42 [ "an": "assertion" ]
            42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn test_or_pattern() {
    let envelope = Envelope::new(42);

    let pattern = Pattern::or(vec![
        Pattern::number_greater_than(40),
        Pattern::text("foo"),
    ]);
    assert!(pattern.matches(&envelope));

    let pattern = Pattern::or(vec![Pattern::text("bar"), Pattern::text("baz")]);
    assert!(!pattern.matches(&envelope));
}

#[test]
fn test_wrapped_pattern() {
    let envelope = Envelope::new(42).wrap_envelope();

    assert!(Pattern::any_wrapped().matches(&envelope));
    assert!(Pattern::wrapped(Pattern::number(42)).matches(&envelope));
    assert!(!Pattern::wrapped(Pattern::number(43)).matches(&envelope));

    let paths: Vec<Path> = Pattern::wrapped(Pattern::number(42))
        .paths(&envelope)
        .into_iter()
        .collect();
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].len(), 2);

    print_paths(&paths);
}

#[test]
fn test_assertion_pattern() {
    let envelope_without_assertions = Envelope::new("Alice");
    assert!(!Pattern::any_assertion().matches(&envelope_without_assertions));

    let envelope_with_assertions = envelope_without_assertions
        .add_assertion("knows", "Bob")
        .add_assertion("worksWith", "Charlie");

    let paths = Pattern::any_assertion()
        .paths(&envelope_with_assertions)
        .into_iter()
        .collect::<Vec<_>>();

    print_paths(&paths);
}

#[test]
fn test_assertion_predicate_pattern() {
    let envelope = Envelope::new("Alice")
        .add_assertion("knows", "Bob")
        .add_assertion("knows", "Charlie")
        .add_assertion("worksWith", "David");

    let pattern = Pattern::assertion_with_predicate(Pattern::text("knows"));
    let paths: Vec<Path> = pattern.paths(&envelope).into_iter().collect();
    print_paths(&paths);
}

#[test]
fn test_credential() {
    let credential = credential();
    println!("{}", credential.tree_format());

    let paths: Vec<Path> = Pattern::wrapped(Pattern::any())
        .paths(&credential)
        .into_iter()
        .collect();
    println!("{}", paths[0][1].tree_format());

    print_paths(&paths);
}
