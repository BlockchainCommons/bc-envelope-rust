mod common;
use bc_envelope::prelude::*;
use common::test_data::*;

// Format each path element on its own line, each line successively indented by
// 4 spaces.
fn format_path(path: Path) -> String {
    let mut lines = Vec::new();
    for (i, element) in path.iter().enumerate() {
        let indent = " ".repeat(i * 4);
        lines.push(format!("{}{}", indent, element.format_flat()));
    }
    lines.join("\n")
}

fn format_paths(paths: Vec<Path>) -> Vec<String> {
    paths.into_iter().map(format_path).collect()
}

fn print_paths(paths: Vec<Path>) {
    let formatted_paths = format_paths(paths);
    for path in formatted_paths {
        println!("{}", path);
    }
}

#[test]
fn test_bool_pattern() {
    let envelope = Envelope::new(true);

    assert!(Pattern::any_bool().is_match(&envelope));
    assert!(Pattern::bool(true).is_match(&envelope));
    assert!(!Pattern::bool(false).is_match(&envelope));

    let envelope = Envelope::new(42);
    assert!(!Pattern::any_bool().is_match(&envelope));
    assert!(!Pattern::bool(true).is_match(&envelope));
    assert!(!Pattern::bool(false).is_match(&envelope));
}

#[test]
fn test_number_pattern() {
    let envelope = Envelope::new(42);

    assert!(Pattern::any_number().is_match(&envelope));
    assert!(Pattern::number(42).is_match(&envelope));
    assert!(!Pattern::number(43).is_match(&envelope));
    assert!(Pattern::number_range(40..=50).is_match(&envelope));
    assert!(!Pattern::number_range(43..=50).is_match(&envelope));
    assert!(Pattern::number_greater_than(41).is_match(&envelope));
    assert!(!Pattern::number_greater_than(42).is_match(&envelope));
    assert!(Pattern::number_less_than(43).is_match(&envelope));
    assert!(!Pattern::number_less_than(42).is_match(&envelope));
    assert!(Pattern::number_greater_than_or_equal(42).is_match(&envelope));
    assert!(!Pattern::number_greater_than_or_equal(43).is_match(&envelope));

    let envelope = Envelope::new("string");
    assert!(!Pattern::any_number().is_match(&envelope));
    assert!(!Pattern::number(42).is_match(&envelope));
}

#[test]
fn test_text_pattern() {
    let envelope = Envelope::new("hello");

    assert!(Pattern::any_text().is_match(&envelope));
    assert!(Pattern::text("hello").is_match(&envelope));
    assert!(!Pattern::text("world").is_match(&envelope));

    let regex = regex::Regex::new(r"^h.*o$").unwrap();
    assert!(Pattern::text_regex(regex).is_match(&envelope));

    let envelope = Envelope::new(42);
    assert!(!Pattern::any_text().is_match(&envelope));
    assert!(!Pattern::text("hello").is_match(&envelope));
}

#[test]
fn test_and_pattern() {
    let envelope = Envelope::new(42);

    let pattern = Pattern::and(vec![
        Pattern::number_greater_than(40),
        Pattern::number_less_than(50),
    ]);
    assert!(pattern.is_match(&envelope));

    let pattern = Pattern::and(vec![Pattern::number(42), Pattern::text("foo")]);
    assert!(!pattern.is_match(&envelope));
}

#[test]
fn test_or_pattern() {
    let envelope = Envelope::new(42);

    let pattern = Pattern::or(vec![
        Pattern::number_greater_than(40),
        Pattern::text("foo"),
    ]);
    assert!(pattern.is_match(&envelope));

    let pattern = Pattern::or(vec![Pattern::text("bar"), Pattern::text("baz")]);
    assert!(!pattern.is_match(&envelope));
}

#[test]
fn test_wrapped_pattern() {
    let envelope = Envelope::new(42).wrap_envelope();

    assert!(Pattern::any_wrapped().is_match(&envelope));
    assert!(Pattern::wrapped(Pattern::number(42)).is_match(&envelope));
    assert!(!Pattern::wrapped(Pattern::number(43)).is_match(&envelope));

    let paths: Vec<Path> = Pattern::wrapped(Pattern::number(42))
        .paths(&envelope)
        .into_iter()
        .collect();
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].len(), 2);

    print_paths(paths);
}

#[test]
fn test_assertion_pattern() {
    let envelope_without_assertions = Envelope::new("Alice");
    assert!(!Pattern::any_assertion().is_match(&envelope_without_assertions));

    let envelope_with_assertions = envelope_without_assertions
        .add_assertion("knows", "Bob")
        .add_assertion("worksWith", "Charlie");

    let paths = Pattern::any_assertion()
        .paths(&envelope_with_assertions)
        .into_iter()
        .collect::<Vec<_>>();

    print_paths(paths);
}

#[test]
fn test_assertion_predicate_pattern() {
    let envelope = Envelope::new("Alice")
        .add_assertion("knows", "Bob")
        .add_assertion("knows", "Charlie")
        .add_assertion("worksWith", "David");

    let pattern = Pattern::assertion_with_predicate(Pattern::text("knows"), Selector::Object);
    let paths: Vec<Path> = pattern.paths(&envelope).into_iter().collect();
    print_paths(paths);
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

    print_paths(paths);
}
