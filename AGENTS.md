⚠️ NOTE: Reading this *entire* file is REQUIRED. Do a `wc -l <path>` to get the number of lines, then fetch the entire file.

# Gordian Envelope Guidelines

## Project Overview

This crate is the Rust reference implementation of [Gordian Envelope](https://www.blockchaincommons.com/introduction/Envelope-Intro/), a structured format for hierarchical binary data focused on privacy. Envelopes are designed to facilitate "smart documents" with features including: representation of various semantic structures, built-in Merkle-like digest tree, deterministic representation using CBOR, and selective encryption or elision of specific document parts without invalidating the structure or signatures.

## Development Environment

### Build/Test Commands

```bash
# Build the crate
cargo build

# Run all tests
cargo test

# Run tests without doc tests
cargo test --all-targets

# Run specific tests with specific features
cargo test --lib --all-features -- module::tests::test_name --exact --show-output

# Run doctests
cargo test --doc
cargo test --doc --all-features -- module::sub_module::Item::test_name --show-output

# Check code quality
cargo clippy -- -D warnings

# Build documentation
cargo doc --no-deps --target-dir cargo-docs
```

### Development Guidelines

- **Production quality** - Write real-world production-quality code
- **Clean code** - Fix all compiler errors and Clippy lints
- **Security focus** - Cryptographic operations must adhere to best practices and be thoroughly tested

### Testing

- Don't mark tasks as complete until all tests pass
- Security-critical components require comprehensive test coverage

## Important Dependencies

### `docs` Directory

🚨 Many Gordian Envelope extensions are described in specifications found in the `bc-research` repository, a subset of which is in `docs/bc-research`. The docs directory also includes the base Gordian Envelope and dCBOR specifications. Always consult the `docs` directory for detailed documentation on extensions such as encryption, signatures, compression, known values, expressions, and more.

### `dcbor` Repository

This repository relies on the `dcbor` crate for deterministic CBOR serialization, which is essential for the consistent representation of Gordian Envelopes.

### `bc-components` Repository

This repository uses cryptographic types and operations defined in the `bc-components` crate, such as `Digest`, signing keys, encryption keys, and other cryptographic primitives.

## Core Reference: Gordian Envelope Format

### Key Data Types

| Type            | Description                                                                 |
| --------------- | --------------------------------------------------------------------------- |
| `Envelope`      | The main container type representing a Gordian Envelope                     |
| `Assertion`     | A predicate-object relationship attached to an envelope subject             |
| `ObscureAction` | Enum specifying whether to elide, encrypt, or compress parts of an envelope |

### Reference Materials

These documents are essential for understanding the Gordian Envelope format and its extensions. There are other documents in the `docs/bc-research` directory that may be useful.

🚨 **NOTE**: The IETF Internet Draft is the primary specification for the base Gordian Envelope format. The `bc-research` repository contains specifications for various extensions.

| Title                         | Description                                   | Location                                                                                                                         |
| ----------------------------- | --------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| Gordian Envelope IETF Draft   | Primary specification for the envelope format | [docs/draft-mcnally-envelope-09.txt](docs/draft-mcnally-envelope-09.txt)                                                         |
| Determinist CBOR (dCBOR) Spec | Specification for deterministic CBOR encoding | [docs/draft-mcnally-deterministic-cbor-12.txt](docs/draft-mcnally-deterministic-cbor-12.txt)                                     |
| BCR-2024-003                  | Envelope Known Value                          | [docs/bc-research/bcr-2023-003-envelope-known-value.md](docs/bc-research/bcr-2023-003-envelope-known-value.md)                   |
| BCR-2024-004                  | Envelope Symmetric Encryption                 | [docs/bc-research/bcr-2023-004-envelope-symmetric-encryption.md](docs/bc-research/bcr-2023-004-envelope-symmetric-encryption.md) |
| BCR-2024-005                  | Envelope Compression                          | [docs/bc-research/bcr-2023-005-envelope-compression.md](docs/bc-research/bcr-2023-005-envelope-compression.md)                   |
| BCR-2024-006                  | Envelope Attachment                           | [docs/bc-research/bcr-2023-006-envelope-attachment.md](docs/bc-research/bcr-2023-006-envelope-attachment.md)                     |
| BCR-2024-012                  | Envelope Expression                           | [docs/bc-research/bcr-2023-012-envelope-expression.md](docs/bc-research/bcr-2023-012-envelope-expression.md)                     |
| BCR-2024-013                  | Envelope Crypto                               | [docs/bc-research/bcr-2023-013-envelope-crypto.md](docs/bc-research/bcr-2023-013-envelope-crypto.md)                             |
| BCR-2024-003                  | Envelope (Full Specification)                 | [docs/bc-research/bcr-2024-003-envelope.md](docs/bc-research/bcr-2024-003-envelope.md)                                           |
| BCR-2024-006                  | Envelope Graph                                | [docs/bc-research/bcr-2024-006-envelope-graph.md](docs/bc-research/bcr-2024-006-envelope-graph.md)                               |
| BCR-2024-007                  | Envelope Decorrelation                        | [docs/bc-research/bcr-2024-007-envelope-decorrelation.md](docs/bc-research/bcr-2024-007-envelope-decorrelation.md)               |

## Documentation Quality Criteria

- **Comprehensive**: All public API elements have documentation
- **Contextual**: Documentation explains both "what" and "why"
- **Practical**: Examples demonstrate real-world usage
- **Consistent**: Uniform style and detail level across the codebase
- **Accessible**: Explanations suitable for developers not familiar with Rust, and Rust engineers not familiar with Gordian Envelope
- **Searchable**: Proper cross-references and keyword usage
- **Validated**: Examples compile and work correctly

## Documentation Testing Guidelines

- **Doc Example Best Practices:**
  - Use appropriate imports in examples, typically `use bc_envelope::prelude::*`
  - Handle errors properly in examples that return `Result`
  - Use `no_run` for examples that can't be directly compiled/run in doc tests. Do *NOT* use `no_run` as a crutch for tests that should be valid but aren't.
  - Use `ignore` for examples that are not meant to be run, but should still compile. This is useful for examples that are fragmentary or too complex to run in a doc test.
  - Check constructors for type initialization in examples - some types may lack `Default` implementation
  - For internal/implementation types that users shouldn't directly interact with, clearly mark them as such in the documentation
  - Before writing examples, refer to unit tests and the `tests/` module to understand how the types are used in practice.
  - In your examples, use `use bc_envelope::prelude::*;` to import all necessary types.
  - Show typical usage patterns for each type, not all possible ways to use it
  - For complex operations like encryption, signatures, and elision, include complete examples that demonstrate the full workflow
  - 🚨 **CRITICAL**: ALL trait implementations (`impl Trait for Type`) MUST have a single-line doc comment explaining the implementation's purpose

## Required Quality Checks

🚨 **CRITICAL**: Always perform these quality checks with EVERY documentation task BEFORE considering it complete:

1. **Fix all doc tests**:
   ```bash
   # Run from the bc-envelope directory, not the workspace root
   cd /path/to/bc-envelope && cargo test --doc
   ```
   Ensure all doc tests pass, and fix any failures immediately.

2. **Fix all Clippy lints**:
   ```bash
   # Run from the bc-envelope directory, not the workspace root
   cd /path/to/bc-envelope && cargo clippy -- -D warnings
   ```
   Address any Clippy warnings introduced by documentation changes.

🔴 **MANDATORY**: YOU MUST RUN THESE CHECKS YOURSELF after making changes, without waiting to be prompted. Documentation is not complete until all tests pass. NEVER mark a task as complete without running and passing these checks.

## Public APIs

### Core Base Module

1. **`EnvelopeError`** (`base/error.rs`) - Error types for Envelope operations
2. **`FormatContext`** (`base/format_context.rs`) - Context for formatting Envelopes
3. **`Assertion`** (`base/assertion.rs`) - A predicate-object relationship
4. **`EnvelopeEncodable`**, **`EnvelopeDecodable`** (`base/envelope_encodable.rs`, `base/envelope_decodable.rs`) - Traits for encoding/decoding Envelopes
5. **`ObscureAction`** (`base/elide.rs`) - Action to take when obscuring envelope parts
6. **`Envelope` and `EnvelopeCase`** (`base/envelope.rs`) - The main Gordian Envelope type and its variants

### Core Functionality

1. `base/wrap.rs` - Functions for wrapping envelopes
2. `base/elide.rs` - Functions for eliding, encrypting, or compressing parts of envelopes
3. `base/assertions.rs` - Functions for working with multiple assertions
4. `base/digest.rs` - Functions for working with the digest tree
5. `base/tree_format.rs` - Functions for tree-formatting envelopes
6. `base/queries.rs` - Functions for querying envelope contents
7. `base/walk.rs` - Functions for walking the envelope hierarchy

### Formatting

1. `format/diagnostic.rs` - Output in CBOR diagnostic format
2. `format/hex.rs` - Output in CBOR hexadecimal format
3. `format/mermaid.rs` - Output in Mermaid format
4. `format/notation.rs` - Output in Envelope Notation format
5. `format/tree.rs` - Output in tree format

### Extension Modules

1. `extension/salt.rs` - Extension for adding salt to decorrelate envelopes
2. `extension/compress.rs` - Extension for compressing envelopes
3. `extension/encrypt.rs` - Extension for encrypting envelopes
4. `extension/sskr.rs` - Extension for Sharded Secret Key Reconstruction
5. `extension/attachment.rs` - Extension for envelope attachments

### Known Values

1. `KnownValue` (`extension/known_values/known_value.rs`) - Predefined known values
2. `KnownValuesStore` (`extension/known_values/known_values_store.rs`) - Store for known values

### Signature Extensions

1. `extension/signature/signature_impl.rs` - Implementation of envelope signatures
2. `SignatureMetadata` (`extension/signature/signature_metadata.rs`) - Metadata for signatures

### Expressions Extensions

1. `Function` (`extension/expressions/function.rs`) - Function for expressions
2. `Parameter` (`extension/expressions/parameter.rs`) - Parameter for expressions
3. `FunctionsStore` (`extension/expressions/functions_store.rs`) - Store for functions
4. `ParametersStore` (`extension/expressions/parameters_store.rs`) - Store for parameters
5. `parameters` (`extension/expressions/parameters.rs`) - Parameter constants and globals
6. `Expression` (`extension/expressions/expression.rs`) - Expression envelope
7. `Request` (`extension/expressions/request.rs`) - Request envelope
8. `Response` (`extension/expressions/response.rs`) - Response envelope
9. `Event` (`extension/expressions/event.rs`) - Event envelope

### Other Extensions

1. `extension/recipient.rs` - Public key encryption
2. `extension/proof.rs` - Inclusion proofs
3. `extension/types.rs` - Type assertions
4. `seal.rs` - Sealing and unsealing convenience functions

## API Design Insights

The following insights about the API design of this crate have been collected during documentation:

1. **Modular Organization**: The crate is organized into modules that group related functionality:
   - **Base Module**: Core envelope functionality including assertions, elision, wrapping, and queries
   - **Extension Module**: Additional features like encryption, compression, signatures, and expressions
   - **Seal Module**: High-level convenience functions that combine multiple operations
   - This organization makes the codebase easier to understand and maintain while keeping related functionality together

2. **Immutability and Mutation Model**: Envelopes are immutable. Operations that "modify" an envelope actually create a new envelope. This is enforced by the API design, which returns new envelopes rather than modifying existing ones.

2. **Reference Counting**: Envelope instances use internal reference counting (either `Rc` or `Arc` depending on features) to efficiently share data structures in memory, making cloning operations inexpensive.

3. **Extension Model**: The base envelope format is extended through feature-gated modules that add functionality like encryption, compression, signatures, etc. This keeps the core small while allowing optional advanced features.

4. **Flexible Type Conversion**: The crate makes extensive use of Rust's type conversion traits:
   - `EnvelopeEncodable` trait for converting values into envelopes
   - `TryFrom<Envelope>` implementations for extracting values from envelopes
   - These allow for fluent, type-safe conversions between Rust native types and envelope structures

5. **CBOR Representation**: All envelopes have a deterministic CBOR representation using the dCBOR spec, which enables consistent serialization across platforms and languages.

6. **Digest Tree**: Every envelope element maintains a cryptographic digest, creating a Merkle-like tree that ensures integrity verification of the entire structure or specific parts.

7. **Structural Variants**: The `EnvelopeCase` enum provides different structural variants (leaf, node, assertion, elided, etc.) that serve specific purposes while maintaining a consistent API. You do NOT need to use the `EnvelopeCase` enum directly in most cases, as the API provides higher-level abstractions for common operations.

8. **Elision and Privacy**: A key feature is the ability to selectively elide, encrypt, or compress parts of an envelope using the `ObscureAction` enum, while maintaining the digest tree structure.

9. **Format Context**: The `FormatContext` provides a rich mechanism for human-readable representation of envelopes, with support for annotating CBOR values, tags, functions, and parameters.

10. **Cryptographic Operations**: The envelope supports various cryptographic operations including signing, encryption, and key recovery through its extension modules.

11. **Expression Model**: Envelopes can represent function calls, requests, and responses through the expressions extension, enabling a protocol for computation and communication.

12. **Assertions Management**: The API provides rich functionality for managing assertions on envelopes, including conditional adding, optional adding, removing, and replacing assertions. These operations maintain the envelope's immutability model.

13. **Semantic vs. Structural Equality**: The API distinguishes between semantic equivalence (same content) and structural identity (same content and structure):
    - `is_equivalent_to()` checks if envelopes contain the same information (O(1) complexity)
    - `is_identical_to()` checks if envelopes have identical structure (O(m+n) complexity)
    - This enables precise comparison based on different requirements

14. **Flexible Query and Extraction API**: The crate provides a rich set of methods for querying envelope contents and extracting typed data:
    - **Multi-level query methods** - from low-level (`subject()`, `assertions()`) to high-level (`extract_object_for_predicate<T>()`)
    - **Type-safe extraction** - The Rust type system ensures type safety when extracting data from envelopes
    - **Predicate-based queries** - Common pattern of finding assertions based on their predicates
    - **Progressive error handling** - Different levels of error reporting based on method granularity

15. **Visitor Pattern for Traversal**: The crate implements a visitor pattern for traversing the envelope hierarchy:
    - **Dual traversal modes** - Structure-based traversal includes all elements while tree-based traversal focuses on semantic content
    - **Edge-labeled traversal** - Each element's relationship to its parent is labeled by edge type during traversal
    - **Context passing** - Visitors can accumulate state or pass context down the hierarchy
    - **Immutable traversal** - The visitor pattern respects the immutable nature of envelopes

16. **Privacy-Enhancing Transformations**: The crate provides several ways to transform envelopes while preserving their digests:
    - **Elision** - Remove content while preserving digests to enable selective disclosure
    - **Decorrelation through Salt** - Add random data to prevent correlation of structurally similar envelopes
    - **Compression** - Reduce size while maintaining digest compatibility
    - **Encryption** - Encrypt content with symmetric keys while preserving digests
    - **Subject-Targeted Transformations** - Apply transformations only to subjects to preserve assertions structure

17. **Social Recovery Support**: The crate supports social recovery of encrypted envelopes through SSKR (Sharded Secret Key Reconstruction):
    - **Threshold Sharing** - Split encryption keys into shares requiring a threshold to reconstruct
    - **Group Structure** - Organize shares into groups with their own thresholds
    - **Share Transport** - Shares are attached to copies of the encrypted envelope for distribution
    - **Private Recovery** - Recovery only requires the shares and not the original key holder

18. **Vendor Extensions**: The crate includes a standardized system for vendor-specific extensions:
    - **Attachments** - Add vendor-specific data to envelopes without affecting the core structure
    - **Vendor Identification** - Required vendor identifiers prevent collisions between extensions
    - **Format Documentation** - Optional conformsTo URIs document the format of attachments
    - **Attachment Queries** - Dedicated methods for finding attachments by vendor and format

19. **Expression System**: The crate provides a rich system for representing and evaluating expressions within envelopes, which is used by the Gordian Sealed Transaction Protocol (GSTP):
    - **CBOR-Tagged Elements** - Functions and parameters use CBOR tags (#6.40006 and #6.40007) to distinguish them from regular envelope content
    - **Dual Identification System** - Both functions and parameters can be identified by either numeric IDs (for well-known/standardized items) or string names (for application-specific items)
    - **Static vs Dynamic Definitions** - Support for both compile-time (static) and runtime (dynamic) definition of functions and parameters
    - **Composable Expressions** - Expressions can be nested as arguments to other expressions, enabling function composition
    - **Natural Envelope Mapping** - The expression structure maps naturally to envelopes: functions as subjects, parameters as predicates, and arguments as objects

20. **Recipient-Based Encryption**: The crate implements a powerful public key encryption system for sharing envelope content with multiple recipients:
    - **Two-Layer Encryption** - Uses a hybrid approach where a random symmetric key (content key) encrypts the envelope's subject, and then the content key is encrypted to each recipient's public key
    - **Multi-Recipient Support** - A single envelope can be encrypted to multiple recipients, each of whom can independently decrypt it without exposing their identity to others
    - **Public/Private Key Separation** - Clean separation between the `Encrypter` trait (using public keys for encryption) and the `Decrypter` trait (using private keys for decryption)
    - **Integrated Signature Support** - Can combine signatures with recipient encryption for authenticated encrypted messages
    - **Fluent API** - Provides both complete operations (`encrypt_to_recipient`) and modular steps (`encrypt_subject` followed by `add_recipient`)

21. **Inclusion Proofs**: The crate includes a system for proving the existence of elements within envelopes without revealing their entire contents:
    - **Merkle-Tree Leveraging** - Builds on the envelope's digest tree structure to create minimal proofs that verify specific elements' existence
    - **Selective Disclosure** - Allows holders to selectively reveal only specific elements while keeping the rest of the envelope private
    - **Proof Generation/Verification Separation** - Clear separation between proof generation (by the holder) and verification (by a third party)
    - **Privacy Enhancement through Salting** - When combined with salting, prevents correlation attacks that try to guess elided content
    - **Set-Based and Single-Element APIs** - Supports proving both individual elements and sets of elements with consistent interfaces
    - **Root Digest Trust Model** - Verifiers only need to trust a root digest, not the entire envelope, enabling efficient verification

22. **Type System**: The crate provides a semantic typing mechanism for envelopes based on the `isA` predicate:
    - **RDF-Inspired Design** - Uses the `isA` known value (similar to RDF's `rdf:type`) to declare envelope types
    - **Type Verification** - Offers methods to check if an envelope has a specific type before processing
    - **Multiple Types** - Supports assigning multiple types to a single envelope for flexible classification
    - **Domain Object Mapping** - Facilitates mapping between domain objects and envelopes through type validation
    - **Two Type Check Modes** - Supports checking against both string/custom types and standard Known Value types
    - **Known Value Integration** - Leverages the Known Values registry for standardized type definitions

23. **Operation Composition**: The crate includes convenience methods that combine multiple operations:
    - **Sealing and Unsealing** - The `seal` and `unseal` methods combine signing and encryption (or decryption and verification) into a single operation
    - **Symmetric and Asymmetric Cryptography** - Allows seamless combination of symmetric content key encryption with asymmetric public key encryption
    - **Fluent API Chaining** - All methods return new envelopes, allowing operations to be chained in a clear, readable manner
