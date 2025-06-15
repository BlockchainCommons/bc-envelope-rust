mod common;

use bc_envelope::prelude::*;
use bc_envelope::pattern::Greediness;

fn nested_wrapped_number(levels: usize) -> Envelope {
    let mut env = Envelope::new(42);
    for _ in 0..levels {
        env = env.wrap_envelope();
    }
    env
}

#[test]
fn repeat_star_greedy() {
    let env = nested_wrapped_number(4);
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), 0..=usize::MAX, Greediness::Greedy),
        Pattern::any_number(),
    ]);
    assert!(pat.matches(&env));
}

#[test]
fn repeat_star_lazy() {
    let env = nested_wrapped_number(4);
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), 0..=usize::MAX, Greediness::Lazy),
        Pattern::any_number(),
    ]);
    assert!(pat.matches(&env));
}

#[test]
fn repeat_star_possessive() {
    let env = nested_wrapped_number(4);
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), 0..=usize::MAX, Greediness::Possessive),
        Pattern::any_number(),
    ]);
    assert!(pat.matches(&env));
}

#[test]
fn repeat_plus_greedy() {
    let env = nested_wrapped_number(3);
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), 1..=usize::MAX, Greediness::Greedy),
        Pattern::any_number(),
    ]);
    assert!(pat.matches(&env));
}

#[test]
fn repeat_plus_lazy() {
    let env = nested_wrapped_number(3);
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), 1..=usize::MAX, Greediness::Lazy),
        Pattern::any_number(),
    ]);
    assert!(pat.matches(&env));
}

#[test]
fn repeat_plus_possessive() {
    let env = nested_wrapped_number(3);
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), 1..=usize::MAX, Greediness::Possessive),
        Pattern::any_number(),
    ]);
    assert!(pat.matches(&env));
}

#[test]
fn repeat_question_greedy() {
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), 0..=1, Greediness::Greedy),
        Pattern::any_number(),
    ]);
    assert!(pat.matches(&Envelope::new(42)));
    assert!(pat.matches(&nested_wrapped_number(1)));
}

#[test]
fn repeat_question_lazy() {
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), 0..=1, Greediness::Lazy),
        Pattern::any_number(),
    ]);
    assert!(pat.matches(&Envelope::new(42)));
    assert!(pat.matches(&nested_wrapped_number(1)));
}

#[test]
fn repeat_question_possessive() {
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), 0..=1, Greediness::Possessive),
        Pattern::any_number(),
    ]);
    assert!(pat.matches(&Envelope::new(42))); 
    assert!(pat.matches(&nested_wrapped_number(1))); 
}

#[test]
fn repeat_range_greedy() {
    let env = nested_wrapped_number(3);
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), 2..=3, Greediness::Greedy),
        Pattern::any_number(),
    ]);
    assert!(pat.matches(&env));
}

#[test]
fn repeat_range_lazy() {
    let env = nested_wrapped_number(3);
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), 2..=3, Greediness::Lazy),
        Pattern::any_number(),
    ]);
    assert!(pat.matches(&env));
}

#[test]
fn repeat_range_possessive() {
    let env = nested_wrapped_number(4);
    let pat = Pattern::sequence(vec![
        Pattern::repeat(Pattern::wrapped(), 2..=3, Greediness::Possessive),
        Pattern::any_number(),
    ]);
    assert!(!pat.matches(&env));
}
