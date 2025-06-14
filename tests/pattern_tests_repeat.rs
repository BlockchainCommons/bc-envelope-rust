mod common;
use bc_envelope::prelude::*;

#[test]
fn optional_wrapper() {
    let inner = Envelope::new("data");
    let wrapped = inner.clone().wrap_envelope();

    let pat = Pattern::sequence(vec![
        Pattern::repeat_greedy(Pattern::any_wrapped(), 0..=1),
        Pattern::subject(),
    ]);

    assert!(pat.matches(&inner));
    assert!(pat.matches(&wrapped));

    // shortest path when unwrapped
    assert_eq!(pat.paths(&inner)[0].len(), 1);
    // wrapped path has two elements
    assert_eq!(pat.paths(&wrapped)[0].len(), 2);
}

#[test]
fn plus_lazy_vs_greedy() {
    let env = Envelope::new("x").wrap_envelope().wrap_envelope();

    let greedy = Pattern::sequence(vec![
        Pattern::repeat_greedy(Pattern::any_wrapped(), 1..=10),
        Pattern::subject(),
    ]);
    let lazy = Pattern::sequence(vec![
        Pattern::repeat_lazy(Pattern::any_wrapped(), 1..=10),
        Pattern::subject(),
    ]);

    assert_eq!(greedy.paths(&env)[0].len(), 3); // two wrappers + subject
    assert_eq!(lazy.paths(&env)[0].len(), 2);   // one wrapper + subject
}
