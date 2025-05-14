use anyhow::{ bail, Result };
use bc_components::{ KeyDerivationMethod, SymmetricKey, EncryptedKey };
use crate::{ Envelope, Error };
use known_values;

impl Envelope {
    pub fn lock_subject(&self, method: KeyDerivationMethod, secret: impl AsRef<[u8]>) -> Self {
        let content_key = SymmetricKey::new();
        // Lock the content key using the specified derivation method
        let encrypted_key = EncryptedKey::lock(method, secret, &content_key);
        // Add a hasSecret assertion with the EncryptedKey
        self.encrypt_subject(&content_key)
            .expect("Failed to encrypt subject")
            .add_assertion(known_values::HAS_SECRET, encrypted_key)
    }

    pub fn unlock_subject(&self, secret: impl AsRef<[u8]>) -> Result<Self> {
        // Find and attempt to unlock each EncryptedKey in hasSecret assertions
        for assertion in self.assertions_with_predicate(known_values::HAS_SECRET) {
            let obj = assertion.as_object().unwrap();
            if !obj.is_obscured() {
                let encrypted_key = obj.extract_subject::<EncryptedKey>()?;
                if let Ok(content_key) = encrypted_key.unlock(secret.as_ref()) {
                    return self.decrypt_subject(&content_key);
                }
            }
        }
        // No matching secret unlock succeeded
        bail!(Error::UnknownSecret)
    }

    pub fn add_secret(
        &self,
        method: KeyDerivationMethod,
        secret: impl AsRef<[u8]>,
        content_key: &SymmetricKey
    ) -> Self {
        // Lock the content key using the specified derivation method
        let encrypted_key = EncryptedKey::lock(method, secret, content_key);
        // Add a hasSecret assertion with the EncryptedKey
        self.add_assertion(known_values::HAS_SECRET, encrypted_key)
    }
}

impl Envelope {
    pub fn lock(&self, method: KeyDerivationMethod, secret: impl AsRef<[u8]>) -> Self {
        self.wrap_envelope().lock_subject(method, secret)
    }

    pub fn unlock(&self, secret: impl AsRef<[u8]>) -> Result<Self> {
        self.unlock_subject(secret)?.unwrap_envelope()
    }
}
