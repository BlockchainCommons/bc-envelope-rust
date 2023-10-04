#[cfg(feature = "encrypt")]
use bc_components::SymmetricKey;

use bc_envelope::prelude::*;

mod common;
use crate::common::test_data::*;

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
    #[cfg(feature = "encrypt")]
    let key = SymmetricKey::new();

    let envelope = Envelope::new(PLAINTEXT_HELLO);
    assert!(!envelope.is_obscured());

    #[cfg(feature = "encrypt")]
    {
        let encrypted = envelope.clone().encrypt_subject(&key).unwrap();
        assert!(encrypted.is_obscured());
    }

    let elided = envelope.clone().elide();
    assert!(elided.is_obscured());

    #[cfg(feature = "compress")]
    {
        let compressed = envelope.clone().compress().unwrap();
        assert!(compressed.is_obscured());
    }
    // ENCRYPTION

    // Cannot encrypt an encrypted envelope.
    //
    // If allowed, would result in an envelope with the same digest but
    // double-encrypted, possibly with a different key, which is probably not what's
    // intended. If you want to double-encrypt then wrap the encrypted envelope first,
    // which will change its digest.
    #[cfg(feature = "encrypt")]
    {
        let encrypted = envelope.clone().encrypt_subject(&key).unwrap();
        encrypted.clone().encrypt_subject(&key).unwrap_err();
    }

    // Cannot encrypt an elided envelope.
    //
    // Elided envelopes have no data to encrypt.
    #[cfg(feature = "encrypt")]
    {
        elided.clone().encrypt_subject(&key).unwrap_err();
    }

    #[cfg(all(feature = "compress", feature = "encrypt"))]
    {
        // OK to encrypt a compressed envelope.
        let compressed = envelope.clone().compress().unwrap();
        let encrypted_compressed = compressed.clone().encrypt_subject(&key).unwrap();
        assert!(encrypted_compressed.is_encrypted());
    }


    // ELISION

    #[cfg(feature = "encrypt")]
    {
        // OK to elide an encrypted envelope.
        let encrypted = envelope.clone().encrypt_subject(&key).unwrap();
        let elided_encrypted = encrypted.clone().elide();
        assert!(elided_encrypted.is_elided());
    }

    // Eliding an elided envelope is idempotent.
    let elided_elided = elided.clone().elide();
    assert!(elided_elided.is_elided());

    #[cfg(feature = "compress")]
    {
        // OK to elide a compressed envelope.
        let compressed = envelope.clone().compress().unwrap();
        let elided_compressed = compressed.clone().elide();
        assert!(elided_compressed.is_elided());
    }

    // COMPRESSION

    // Cannot compress an encrypted envelope.
    //
    // Encrypted envelopes cannot become smaller because encrypted data looks random,
    // and random data is not compressible.
    #[cfg(all(feature = "compress", feature = "encrypt"))]
    {
        let encrypted = envelope.clone().encrypt_subject(&key).unwrap();
        encrypted.compress().unwrap_err();
    }

    // Cannot compress an elided envelope.
    //
    // Elided envelopes have no data to compress.
    #[cfg(feature = "compress")]
    elided.compress().unwrap_err();

    #[cfg(feature = "compress")]
    {
        // Compressing a compressed envelope is idempotent.
        let compressed = envelope.compress().unwrap();
        let compressed_compressed = compressed.compress().unwrap();
        assert!(compressed_compressed.is_compressed());
    }
}
