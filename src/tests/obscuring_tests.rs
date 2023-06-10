use bc_components::SymmetricKey;

use crate::IntoEnvelope;

use super::test_data::PLAINTEXT_HELLO;

/// This tests the transformation of different kinds of "obscured" envelopes into
/// others. Some transformations are allowed, some are idempotent (return the same
/// result), and some throw errors.
///
/// | Operation > | Encrypt | Elide      | Compress   |
/// |:------------|:--------|:-----------|:-----------|
/// | Encrypted   | ERROR   | OK         | ERROR      |
/// | Elided      | ERROR   | IDEMPOTENT | ERROR      |
/// | Compressed  | OK      | OK         | IDEMPOTENT |
///
#[test]
fn test_obscuring() {
    let key = SymmetricKey::new();

    let envelope = PLAINTEXT_HELLO.into_envelope();
    assert!(!envelope.is_obscured());

    let encrypted = envelope.clone().encrypt_subject(&key).unwrap();
    assert!(encrypted.is_obscured());

    let elided = envelope.clone().elide();
    assert!(elided.is_obscured());

    let compressed = envelope.compress().unwrap();
    assert!(compressed.is_obscured());

    // ENCRYPTION

    // Cannot encrypt an encrypted envelope.
    //
    // If allowed, would result in an envelope with the same digest but
    // double-encrypted, possibly with a different key, which is probably not what's
    // intended. If you want to double-encrypt then wrap the encrypted envelope first,
    // which will change its digest.
    encrypted.clone().encrypt_subject(&key).unwrap_err();

    // Cannot encrypt an elided envelope.
    //
    // Elided envelopes have no data to encrypt.
    elided.clone().encrypt_subject(&key).unwrap_err();

    // OK to encrypt a compressed envelope.
    let encrypted_compressed = compressed.clone().encrypt_subject(&key).unwrap();
    assert!(encrypted_compressed.is_encrypted());


    // ELISION

    // OK to elide an encrypted envelope.
    let elided_encrypted = encrypted.clone().elide();
    assert!(elided_encrypted.is_elided());

    // Eliding an elided envelope is idempotent.
    let elided_elided = elided.clone().elide();
    assert!(elided_elided.is_elided());

    // OK to elide a compressed envelope.
    let elided_compressed = compressed.clone().elide();
    assert!(elided_compressed.is_elided());

    // COMPRESSION

    // Cannot compress an encrypted envelope.
    //
    // Encrypted envelopes cannot become smaller because encrypted data looks random,
    // and random data is not compressible.
    encrypted.compress().unwrap_err();

    // Cannot compress an elided envelope.
    //
    // Elided envelopes have no data to compress.
    elided.compress().unwrap_err();

    // Compressing a compressed envelope is idempotent.
    let compressed_compressed = compressed.compress().unwrap();
    assert!(compressed_compressed.is_compressed());
}
