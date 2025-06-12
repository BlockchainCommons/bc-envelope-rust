## Task: Implementation of remaining patterns in the `structure` submodule

- Implement `DigestPattern`, `NodePattern`, and `ObscuredPattern` in the `structure/` submodule.
- Their current implementations must be considered partial and were written and disabled before the rest of the currently-working API. They may still be considered good starting points.
- "Obscured" in Envelope parlance means "elided, encrypted, or compressed," which are all transformations of a branch of the envelope tree that still preserve its digest. Look in the `tests` module for examples of how these types of envelopes are constructed and tested.
- Note that in `pattern_tests.rs`, `format_paths()` prefixes each path elements with the first 8 hex digits of the digest of the envelope. This can be used as a prefix `DigestPattern::HexPrefix`.
- Study `envelope.rs` and `queries.rs` to understand the Envelope API.

## The `pattern` Module

pattern/

- pattern.rs
  - The main entry point for the pattern module, aggregating all patterns.
- matcher.rs
  - The `Matcher` trait, which all patterns implement to provide matching functionality.

## Patterns dealing with Envelope Structure

structure/

- [x] structure_pattern.rs
  - Aggregates all structure patterns
- [ ] digest_pattern.rs
- [ ] node_pattern.rs
- [ ] obscured_pattern.rs
- [x] assertions_pattern.rs
- [x] object_pattern.rs
- [x] predicate_pattern.rs
- [x] subject_pattern.rs
- [x] wrapped_pattern.rs

## Meta-Patterns

meta/

- [x] meta_pattern.rs
  - Aggregates all meta patterns
- [ ] not_pattern.rs
- [ ] repeat_pattern.rs
- [x] and_pattern.rs
- [x] or_pattern.rs
- [x] search_pattern.rs
- [x] sequence_pattern.rs

## Patterns dealing with Leaf Node (CBOR) values

leaf/

- [x] leaf_pattern.rs
  - Aggregates all leaf patterns
- [ ] array_pattern.rs
- [ ] byte_string_pattern.rs
- [ ] cbor_pattern.rs
- [ ] known_value_pattern.rs
- [ ] map_pattern.rs
- [ ] null_pattern.rs
- [ ] tag_pattern.rs
- [x] bool_pattern.rs
- [x] number_pattern.rs
- [x] text_pattern.rs
