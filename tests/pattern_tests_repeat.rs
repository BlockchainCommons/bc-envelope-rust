mod common;
use bc_envelope::prelude::*;

use crate::common::pattern_utils::format_paths;

#[test]
fn optional_wrapper() {
    let inner = Envelope::new("data");
    let wrapped = inner.clone().wrap_envelope();

    println!("=== Tree Format ===");
    println!("inner tree:\n\n{}\n", inner.tree_format());
    println!("wrapped tree:\n\n{}\n", wrapped.tree_format());

    let pat = Pattern::sequence(vec![
        Pattern::repeat_greedy(Pattern::wrapped(), 0..=1),
        Pattern::subject(),
    ]);

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
fn plus_lazy_vs_greedy() {
    let env = Envelope::new("x").wrap_envelope().wrap_envelope();

    let greedy = Pattern::sequence(vec![
        Pattern::repeat_greedy(Pattern::wrapped(), 1..=10),
        Pattern::subject(),
    ]);
    let lazy = Pattern::sequence(vec![
        Pattern::repeat_lazy(Pattern::wrapped(), 1..=10),
        Pattern::subject(),
    ]);

    assert_eq!(greedy.paths(&env)[0].len(), 3); // two wrappers + subject
    assert_eq!(lazy.paths(&env)[0].len(), 2); // one wrapper + subject
}
