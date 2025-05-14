# Blockchain Commons Gordian Envelope for Rust

### _by Wolf McNally and Blockchain Commons_

---

## Introduction

[Gordian Envelope](https://www.blockchaincommons.com/introduction/Envelope-Intro/) is a structured format for hierarchical binary data focused on privacy. The Rust implementation provides a feature-rich, production-ready reference implementation.

Envelopes are designed to facilitate "smart documents" with a number of unique features:

- **Hierarchical structure**: Easy representation of a variety of semantic structures, from simple key-value pairs to complex property graphs
- **Merkle-like digest tree**: Built-in integrity verification at any level of the structure
- **Deterministic representation**: Uses CBOR with deterministic encoding rules for consistent serialization
- **Privacy-focused**: The holder of a document can selectively encrypt or elide specific parts without invalidating the structure, signatures, or digest tree
- **Progressive trust**: Holders can reveal information incrementally to build trust with verifiers

## Getting Started

```toml
[dependencies]
bc-envelope = "0.28.0"
```

Basic usage examples:

```rust
use bc_envelope::prelude::*;

// Create a simple envelope with a string subject
let envelope = Envelope::new("Hello, world!");

// Add assertions to an envelope
let envelope = envelope
    .add_assertion("created", "2025-04-03")
    .add_assertion("author", "Alice");

// Selectively elide information
let elided = envelope.elide(|e| !e.extract_object_for_predicate::<String>("author").unwrap().eq("Alice"));

// Sign an envelope
let signed = envelope.add_signature(&private_key);

// Verify a signature
let verified = signed.verify_signature_from(&public_key)?;
```

## Features and Components

Gordian Envelope includes several powerful features, all configurable through Cargo features:

| Feature       | Description                                           |
| ------------- | ----------------------------------------------------- |
| `base`        | Core envelope functionality and digest tree           |
| `encrypt`     | Symmetric encryption of envelope parts                |
| `signature`   | Digital signatures for authenticity and integrity     |
| `compress`    | Compression for efficient storage/transmission        |
| `known_value` | Registry of well-known semantic values                |
| `expression`  | Support for representing function calls and requests  |
| `recipient`   | Public key encryption to multiple recipients          |
| `salt`        | Decorrelation of structurally similar envelopes       |
| `sskr`        | Sharded Secret Key Reconstruction for social recovery |
| `attachment`  | Vendor-specific extensions mechanism                  |

By default, all features are enabled. You can select a subset for your specific needs.

## Specification

Gordian Envelope is formally specified in the [IETF Internet Draft](https://datatracker.ietf.org/doc/draft-mcnally-envelope/), which is currently in the community review stage.

Extensions to the base specification are documented in the [Blockchain Commons Research repository](https://github.com/BlockchainCommons/bc-research):

- [BCR-2023-003](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2023-003-envelope-known-value.md): Envelope Known Value
- [BCR-2023-004](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2023-004-envelope-symmetric-encryption.md): Envelope Symmetric Encryption
- [BCR-2023-005](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2023-005-envelope-compression.md): Envelope Compression
- [BCR-2023-006](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2023-006-envelope-attachment.md): Envelope Attachment
- [BCR-2023-012](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2023-012-envelope-expression.md): Envelope Expression
- [BCR-2023-013](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2023-013-envelope-crypto.md): Envelope Crypto
- [BCR-2024-003](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2024-003-envelope.md): Envelope (Full Specification)
- [BCR-2024-006](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2024-006-envelope-graph.md): Envelope Graph
- [BCR-2024-007](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2024-007-envelope-decorrelation.md): Envelope Decorrelation

## Gordian Principles

Gordian Envelope implements the [Gordian Principles](https://github.com/BlockchainCommons/Gordian#gordian-principles):

- **Independence**: Holders control their own data and can selectively disclose information without relying on third parties
- **Privacy**: Built-in support for elision, encryption, and decorrelation allows precise information control
- **Resilience**: Support for social recovery (SSKR), signatures, and encryption provides robust security
- **Openness**: Fully open-source, well-documented, with open standards and test vectors

## Status - Community Review

Gordian Envelope is currently in community review. We appreciate your testing and feedback on:

- API usability
- Integration with your use cases
- Performance
- Bug reports

Comments can be posted [to the Gordian Developer Community](https://github.com/BlockchainCommons/Gordian-Developer-Community/discussions/116).

Because this library is still in community review, it should not be used for production tasks until it has received further testing and auditing.

See [Blockchain Commons' Development Phases](https://github.com/BlockchainCommons/Community/blob/master/release-path.md).

## Dependencies

- [dcbor](https://crates.io/crates/dcbor): Deterministic CBOR for consistent serialization
- [bc-crypto](https://crates.io/crates/bc-crypto): Cryptographic primitives
- [bc-components](https://crates.io/crates/bc-components): Common cryptographic components
- [bc-ur](https://crates.io/crates/bc-ur): Uniform Resources for encoding and decoding

## Other Implementations

Gordian Envelope is also available in:

- [Swift](https://github.com/BlockchainCommons/BCSwiftEnvelope)
- [Command-line tool](https://crates.io/crates/bc-envelope-cli)

## Contributing

We encourage public contributions through issues and pull requests! Please review [CONTRIBUTING.md](./CONTRIBUTING.md) for details on our development process. All contributions to this repository require a GPG signed [Contributor License Agreement](./CLA.md).

## Financial Support

Gordian Envelope is a project of [Blockchain Commons](https://www.blockchaincommons.com/), a "not-for-profit" social benefit corporation committed to open source & open development. Our work is funded entirely by donations and collaborative partnerships with people like you. Every contribution will be spent on building open tools, technologies, and techniques that sustain and advance blockchain and internet security infrastructure and promote an open web.

To financially support further development of Gordian Envelope and other projects, please consider becoming a Patron of Blockchain Commons through ongoing monthly patronage as a [GitHub Sponsor](https://github.com/sponsors/BlockchainCommons). You can also support Blockchain Commons with bitcoins at our [BTCPay Server](https://btcpay.blockchaincommons.com/).

## Discussions

The best place to talk about Blockchain Commons and its projects is in our GitHub Discussions areas:

- [Gordian Developer Community](https://github.com/BlockchainCommons/Gordian-Developer-Community/discussions): For developers working with Gordian specifications
- [Blockchain Commons Discussions](https://github.com/BlockchainCommons/Community/discussions): For general Blockchain Commons topics

## Credits

The following people directly contributed to this repository:

| Name              | Role                | Github                                           | Email                               | GPG Fingerprint                                   |
| ----------------- | ------------------- | ------------------------------------------------ | ----------------------------------- | ------------------------------------------------- |
| Christopher Allen | Principal Architect | [@ChristopherA](https://github.com/ChristopherA) | <ChristopherA@LifeWithAlacrity.com> | FDFE 14A5 4ECB 30FC 5D22 74EF F8D3 6C91 3574 05ED |
| Wolf McNally      | Contributor         | [@WolfMcNally](https://github.com/wolfmcnally)   | <Wolf@WolfMcNally.com>              | 9436 52EE 3844 1760 C3DC 3536 4B6C 2FCF 8947 80AE |

## Responsible Disclosure

We want to keep all our software safe for everyone. If you have discovered a security vulnerability, we appreciate your help in disclosing it to us in a responsible manner. Please see our [security policy](SECURITY.md) for details.

## License

Gordian Envelope is licensed under the BSD-2-Clause-Patent license. See [LICENSE](./LICENSE.md) for details.
