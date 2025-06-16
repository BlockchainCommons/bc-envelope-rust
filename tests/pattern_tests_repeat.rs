mod common;

use bc_envelope::prelude::*;
use indoc::indoc;

use crate::common::pattern_utils::format_paths;

fn fold(string: &str) -> Envelope {
    let chars: Vec<String> = string.chars().map(|c| c.to_string()).collect();
    let mut it = chars.into_iter().enumerate().rev();
    let (index, c) = it.next().unwrap();
    let mut env = Envelope::new_assertion(index, c);
    for (index, c) in it {
        let obj = Envelope::new(c.clone())
            .add_assertion_envelope(env)
            .unwrap();
        env = Envelope::new_assertion(index, obj);
    }
    Envelope::unit().add_assertion_envelope(env).unwrap()
}

fn unfold(env: impl AsRef<Envelope>) -> String {
    let mut result = String::new();
    let mut env = Some(env.as_ref().clone());
    while let Some(e) = env {
        if e.is_assertion() {
            let object = e.as_object().unwrap();
            let c: String = object.extract_subject().unwrap();
            result.push_str(&c);
            env = object.assertions().first().cloned();
        } else {
            env = e.assertions().first().cloned();
        }
    }
    result
}

#[test]
fn test_fold() {
    bc_envelope::register_tags();

    let s = "hello";
    let folded = fold(s);

    #[rustfmt::skip]
    let expected = indoc! {r#"
        '' [
            0: "h" [
                1: "e" [
                    2: "l" [
                        3: "l" [
                            4: "o"
                        ]
                    ]
                ]
            ]
        ]
    "#}.trim();
    assert_actual_expected!(folded.format(), expected);

    #[rustfmt::skip]
    let expected =  indoc! {r#"
        b229d3cb NODE
            934312d6 subj ''
            1b47f7a1 ASSERTION
                6e340b9c pred 0
                dc1d9ddc obj NODE
                    70a0d519 subj "h"
                    354b5ed3 ASSERTION
                        4bf5122f pred 1
                        a899ff63 obj NODE
                            f9a00f43 subj "e"
                            7e272ce6 ASSERTION
                                dbc1b4c9 pred 2
                                bff05dca obj NODE
                                    63518250 subj "l"
                                    d71e5aaf ASSERTION
                                        084fed08 pred 3
                                        73381991 obj NODE
                                            63518250 subj "l"
                                            7c92231b ASSERTION
                                                e52d9c50 pred 4
                                                2dd41130 obj "o"
    "#}.trim();
    assert_actual_expected!(folded.tree_format(), expected);

    let unfolded = unfold(folded);
    assert_eq!(unfolded, s);
}

#[test]
fn repeat_test() {
    bc_envelope::register_tags();

    let s = "hello";
    let env = fold(s);

    let paths = Pattern::sequence(vec![Pattern::any_assertion()]).paths(&env);
    assert_eq!(unfold(paths[0].last().unwrap()), s);

    let assertion_object_pattern = Pattern::sequence(vec![
        Pattern::any_assertion(),
        Pattern::any_object(),
    ]);

    let paths =
        Pattern::repeat(assertion_object_pattern, 3..=3, Greediness::Greedy)
            .paths(&env);
    assert_eq!(paths.len(), 1);
    println!("{}", format_paths(&paths));

    let path = &paths[0];
    assert_eq!(transpose(path), "hel");
    assert_eq!(unfold(path.last().unwrap()), "lo");
    for element in path {
        println!("{}", unfold(element));
    }
}

#[test]
fn test_repeat_2() {
    let str = "AabBbabB";
    let env = fold(str);

    let seq_a = Pattern::sequence(vec![
        Pattern::assertion_with_object(Pattern::text("A")),
        Pattern::any_object(),
    ]);

    let seq_any = Pattern::sequence(vec![
        Pattern::any_assertion(),
        Pattern::any_object(),
    ]);

    let seq_b = Pattern::sequence(vec![
        Pattern::assertion_with_object(Pattern::text("B")),
        Pattern::any_object(),
    ]);

    let pat = |mode| Pattern::sequence(vec![
        seq_a.clone(),
        Pattern::repeat(seq_any.clone(), .., mode),
        seq_b.clone(),
    ]);

    let paths = pat(Greediness::Greedy).paths(&env);
    println!("\nGreedy:\n{}", format_paths(&paths));

    let paths = pat(Greediness::Lazy).paths(&env);
    println!("\nLazy:\n{}", format_paths(&paths));

    let paths = pat(Greediness::Possessive).paths(&env);
    println!("\nPossessive:\n{}", format_paths(&paths));
}

fn transpose(path: impl AsRef<Path>) -> String {
    path.as_ref().iter().filter_map(|e| e.subject().as_text()).collect::<Vec<_>>().join("")
}

fn wrap_n(mut env: Envelope, n: usize) -> Envelope {
    for _ in 0..n {
        env = env.wrap_envelope();
    }
    env
}

#[test]
fn repeat_any_greedy() {
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), .., Greediness::Greedy),
        Pattern::any_cbor(),
    ]);

    let env = wrap_n(Envelope::new(42), 4);
    let paths = pat.paths(&env);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        3a0b1e87 { { { { 42 } } } }
            75659622 { { { 42 } } }
                81bb1f5e { { 42 } }
                    58b1ac6a { 42 }
                        7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn repeat_any_lazy() {
    let env = wrap_n(Envelope::new(42), 4);
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), .., Greediness::Lazy),
        Pattern::any_cbor(),
    ]);
    let paths = pat.paths(&env);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        3a0b1e87 { { { { 42 } } } }
            75659622 { { { 42 } } }
                81bb1f5e { { 42 } }
                    58b1ac6a { 42 }
                        7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn repeat_any_possessive() {
    let env = wrap_n(Envelope::new(42), 4);
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), .., Greediness::Possessive),
        Pattern::any_cbor(),
    ]);
    let paths = pat.paths(&env);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        3a0b1e87 { { { { 42 } } } }
            75659622 { { { 42 } } }
                81bb1f5e { { 42 } }
                    58b1ac6a { 42 }
                        7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn repeat_some_greedy() {
    let env = wrap_n(Envelope::new(42), 3);
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), 1.., Greediness::Greedy),
        Pattern::any_cbor(),
    ]);
    let paths = pat.paths(&env);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        75659622 { { { 42 } } }
            81bb1f5e { { 42 } }
                58b1ac6a { 42 }
                    7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn repeat_some_lazy() {
    let env = wrap_n(Envelope::new(42), 3);
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), 1.., Greediness::Lazy),
        Pattern::any_cbor(),
    ]);
    let paths = pat.paths(&env);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        75659622 { { { 42 } } }
            81bb1f5e { { 42 } }
                58b1ac6a { 42 }
                    7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn repeat_some_possessive() {
    let env = wrap_n(Envelope::new(42), 3);
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), 1.., Greediness::Possessive),
        Pattern::any_cbor(),
    ]);
    let paths = pat.paths(&env);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        75659622 { { { 42 } } }
            81bb1f5e { { 42 } }
                58b1ac6a { 42 }
                    7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn repeat_optional_greedy() {
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), 0..=1, Greediness::Greedy),
        Pattern::any_cbor(),
    ]);
    let paths = pat.paths(&wrap_n(Envelope::new(42), 0));
    #[rustfmt::skip]
    let expected = indoc! {r#"
        7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);

    let paths = pat.paths(&wrap_n(Envelope::new(42), 1));
    #[rustfmt::skip]
    let expected = indoc! {r#"
        58b1ac6a { 42 }
            7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn repeat_optional_lazy() {
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), 0..=1, Greediness::Lazy),
        Pattern::any_cbor(),
    ]);
    let paths = pat.paths(&wrap_n(Envelope::new(42), 0));
    #[rustfmt::skip]
    let expected = indoc! {r#"
        7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
    let paths = pat.paths(&wrap_n(Envelope::new(42), 1));
    #[rustfmt::skip]
    let expected = indoc! {r#"
        58b1ac6a { 42 }
            7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn repeat_optional_possessive() {
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), 0..=1, Greediness::Possessive),
        Pattern::any_cbor(),
    ]);
    let paths = pat.paths(&wrap_n(Envelope::new(42), 0));
    #[rustfmt::skip]
    let expected = indoc! {r#"
        7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
    let paths = pat.paths(&wrap_n(Envelope::new(42), 1));
    #[rustfmt::skip]
    let expected = indoc! {r#"
        58b1ac6a { 42 }
            7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn repeat_range_greedy() {
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), 2..=3, Greediness::Greedy),
        Pattern::any_cbor(),
    ]);
    let env = wrap_n(Envelope::new(42), 3);
    assert!(pat.matches(&env));
    let paths = pat.paths(&env);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        75659622 { { { 42 } } }
            81bb1f5e { { 42 } }
                58b1ac6a { 42 }
                    7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn repeat_range_lazy() {
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), 2..=3, Greediness::Lazy),
        Pattern::any_cbor(),
    ]);
    let env = wrap_n(Envelope::new(42), 3);
    let paths = pat.paths(&env);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        75659622 { { { 42 } } }
            81bb1f5e { { 42 } }
                58b1ac6a { 42 }
                    7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn repeat_range_possessive() {
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), 2..=3, Greediness::Possessive),
        Pattern::any_cbor(),
    ]);
    let env = wrap_n(Envelope::new(42), 3);
    let paths = pat.paths(&env);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        75659622 { { { 42 } } }
            81bb1f5e { { 42 } }
                58b1ac6a { 42 }
                    7f83f7bd 42
    "#}.trim();
    assert_actual_expected!(format_paths(&paths), expected);
}

#[test]
fn repeat_any_modes() {
    let env = wrap_n(Envelope::new("data"), 2);

    let pat = |mode| {
        Pattern::sequence(vec![
            Pattern::repeat(Pattern::wrapped(), 0.., mode),
            Pattern::wrapped(),
            Pattern::text("data"),
        ])
    };

    let greedy_paths = pat(Greediness::Greedy).paths(&env);
    let lazy_paths = pat(Greediness::Lazy).paths(&env);
    let possessive_paths = pat(Greediness::Possessive).paths(&env);

    assert_eq!(greedy_paths, lazy_paths);
    assert!(possessive_paths.is_empty());

    #[rustfmt::skip]
    let expected = indoc! {r#"
        ee8cade0 { { "data" } }
            febc1555 { "data" }
                e909da9a "data"
    "#}.trim();
    assert_actual_expected!(format_paths(&greedy_paths), expected);
}

#[test]
fn repeat_optional_modes() {
    let env = wrap_n(Envelope::new(42), 1);

    let pat = |mode| {
        Pattern::sequence(vec![
            Pattern::repeat(Pattern::wrapped(), 0..=1, mode),
            Pattern::number(42),
        ])
    };

    let greedy_paths = pat(Greediness::Greedy).paths(&env);
    let expected = indoc! {r#"
        58b1ac6a { 42 }
            7f83f7bd 42
    "#}
    .trim();
    assert_actual_expected!(format_paths(&greedy_paths), expected);

    let lazy_paths = pat(Greediness::Lazy).paths(&env);
    let expected = indoc! {r#"
        58b1ac6a { 42 }
            7f83f7bd 42
    "#}
    .trim();
    assert_actual_expected!(format_paths(&lazy_paths), expected);

    let possessive_paths = pat(Greediness::Possessive).paths(&env);
    let expected = indoc! {r#"
        58b1ac6a { 42 }
            7f83f7bd 42
    "#}
    .trim();
    assert_actual_expected!(format_paths(&possessive_paths), expected);
}

#[test]
fn repeat_some_order() {
    let env = wrap_n(Envelope::new("x"), 2);

    let expected = indoc! {r#"
        06bb2465 WRAPPED
            70b5f17d subj WRAPPED
                5e85370e subj "x"
    "#}
    .trim();
    assert_actual_expected!(env.tree_format(), expected);

    let pat = |mode| {
        Pattern::sequence(vec![
            Pattern::repeat(Pattern::wrapped(), 1.., mode),
            Pattern::subject(),
        ])
    };

    let greedy_paths = pat(Greediness::Greedy).paths(&env);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        06bb2465 { { "x" } }
            70b5f17d { "x" }
                5e85370e "x"
        06bb2465 { { "x" } }
            70b5f17d { "x" }
    "#}.trim();
    assert_actual_expected!(format_paths(&greedy_paths), expected);

    let lazy_paths = pat(Greediness::Lazy).paths(&env);
    #[rustfmt::skip]
    let expected = indoc! {r#"
        06bb2465 { { "x" } }
            70b5f17d { "x" }
        06bb2465 { { "x" } }
            70b5f17d { "x" }
                5e85370e "x"
    "#}.trim();
    assert_actual_expected!(format_paths(&lazy_paths), expected);

    let possessive_paths = pat(Greediness::Possessive).paths(&env);
    let expected = indoc! {r#"
        06bb2465 { { "x" } }
            70b5f17d { "x" }
                5e85370e "x"
    "#}
    .trim();
    assert_actual_expected!(format_paths(&possessive_paths), expected);
}

#[test]
fn repeat_range_order() {
    let env = wrap_n(Envelope::new("x"), 4);

    let pat = |mode| {
        Pattern::sequence(vec![
            Pattern::repeat(Pattern::wrapped(), 2..=3, mode),
            Pattern::subject(),
        ])
    };

    let greedy_paths = pat(Greediness::Greedy).paths(&env);
    let expected = indoc! {r#"
        88e28c8b { { { { "x" } } } }
            79962374 { { { "x" } } }
                06bb2465 { { "x" } }
                    70b5f17d { "x" }
        88e28c8b { { { { "x" } } } }
            79962374 { { { "x" } } }
                06bb2465 { { "x" } }
    "#}
    .trim();
    assert_actual_expected!(format_paths(&greedy_paths), expected);

    let lazy_paths = pat(Greediness::Lazy).paths(&env);
    let expected = indoc! {r#"
        88e28c8b { { { { "x" } } } }
            79962374 { { { "x" } } }
                06bb2465 { { "x" } }
        88e28c8b { { { { "x" } } } }
            79962374 { { { "x" } } }
                06bb2465 { { "x" } }
                    70b5f17d { "x" }
    "#}
    .trim();
    assert_actual_expected!(format_paths(&lazy_paths), expected);

    let possessive_paths = pat(Greediness::Possessive).paths(&env);
    let expected = indoc! {r#"
        88e28c8b { { { { "x" } } } }
            79962374 { { { "x" } } }
                06bb2465 { { "x" } }
                    70b5f17d { "x" }
    "#}
    .trim();
    assert_actual_expected!(format_paths(&possessive_paths), expected);
}
