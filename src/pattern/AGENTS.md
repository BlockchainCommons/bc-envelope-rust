## Task: Organize the pattern module

- Currently all the files in the `pattern` module are at the top level.
- Reorganize them into subdirectories based on their functionality as categorized below.
- The goal is to improve the structure and maintainability of the codebase.
- A number of the files are empty or disabled for compilation. Keep them in this state for later implementation.
- Make sure all tests in `tests/pattern_tests.rs` still pass after the reorganization.

## Top-level

pattern/

- pattern.rs
  - The main entry point for the pattern module, aggregating all patterns.
- matcher.rs
  - The `Matcher` trait, which all patterns implement to provide matching functionality.

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

### Final Status

- [x] All pattern tests (12) continue to pass
- [x] Project builds successfully in debug and release modes
- [x] Fixed visibility warnings by changing aggregator enum visibility from pub(crate) to pub
- [x] Build warnings resolved - only expected "unused code" warnings for new aggregator patterns remain
- [x] Reorganization complete: patterns organized into structure/, leaf/, and meta/ subdirectories
- [x] Individual pattern types marked as pub(crate) for internal module use
- [x] Only Pattern, Matcher, and Path types remain public in main module API
- [x] Unimplemented pattern files remain disabled as requested
