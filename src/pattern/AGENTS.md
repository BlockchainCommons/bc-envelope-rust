## Task: Implementation of remaining patterns in the `leaf` submodule

- The task is to implement the remaining patterns in the `leaf` submodule of the `pattern` module.
- See the list below for the patterns that need to be implemented.
- Use patterns that are already implemented as a reference for your implementation.

## General Guidelines

- Gordian Envelope is a tree structure whose leaves are dCBOR values. The `dcbor` crate in this workspace is used to handle these CBOR values, so you will probably want to familiarize yourself with it and its tests.
- Study `envelope.rs`, `queries.rs`, and `leaf.rs` to understand the Envelope API.
- Note that Envelope APIs that deal with extracting CBOR leaf values are usually prefixed with `extract_`.
- There are also Envelope predicates like `is_*` that can be used to check the type of a leaf value, e.g., `is_text`, `is_number`, `is_bool`, etc.
- Make sure to add or adjust tests for any new functionality you add or change.
- Ensure all crate tests pass. You can skip doc tests initially, so use `cargo test --all-targets` when you want to test the whole crate but not the doc tests.
- Use `cargo test --package bc-envelope --test 'pattern_tests*'` to run the tests for the `pattern` module specifically.
- Make sure all clippy lints are also resolved.

## The `pattern` Module

pattern/

- pattern.rs
  - The main entry point for the pattern module, aggregating all patterns.
- matcher.rs
  - The `Matcher` trait, which all patterns implement to provide matching functionality.

### Patterns dealing with Leaf Node (CBOR) values

leaf/

- [x] leaf_pattern.rs
  - Aggregates all leaf patterns
- [x] array_pattern.rs
- [x] byte_string_pattern.rs
- [x] cbor_pattern.rs
- [x] known_value_pattern.rs
- [ ] map_pattern.rs
- [ ] null_pattern.rs
- [ ] tag_pattern.rs
- [x] bool_pattern.rs
- [x] number_pattern.rs
- [x] text_pattern.rs

### Patterns dealing with Envelope Structure

structure/

- [x] structure_pattern.rs
  - Aggregates all structure patterns
- [x] digest_pattern.rs
- [x] node_pattern.rs
- [x] obscured_pattern.rs
- [x] assertions_pattern.rs
- [x] object_pattern.rs
- [x] predicate_pattern.rs
- [x] subject_pattern.rs
- [x] wrapped_pattern.rs

### Meta-Patterns

meta/

- [x] meta_pattern.rs
  - Aggregates all meta patterns
- [ ] not_pattern.rs
- [ ] repeat_pattern.rs
- [x] and_pattern.rs
- [x] or_pattern.rs
- [x] search_pattern.rs
- [x] sequence_pattern.rs
