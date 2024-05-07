use std::sync::{Once, Mutex};
use paste::paste;

use super::known_values_store::KnownValuesStore;

/// A macro that declares a known value at compile time.
#[macro_export]
macro_rules! known_value_constant {
    ($const_name:ident, $value:expr, $name:expr) => {
        paste! {
            pub const [<$const_name _RAW>]: u64 = $value;
        }
        pub const $const_name: $crate::extension::known_values::KnownValue = $crate::extension::known_values::KnownValue::new_with_static_name($value, $name);
    };
}

// For definitions see: https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2023-002-known-value.md#appendix-a-registry

known_value_constant!(IS_A, 1, "isA");
known_value_constant!(ID, 2, "id");
known_value_constant!(VERIFIED_BY, 3, "verifiedBy");
known_value_constant!(NOTE, 4, "note");
known_value_constant!(HAS_RECIPIENT, 5, "hasRecipient");
known_value_constant!(SSKR_SHARE, 6, "sskrShare");
known_value_constant!(CONTROLLER, 7, "controller");
known_value_constant!(PUBLIC_KEYS, 8, "publicKeys");
known_value_constant!(DEREFERENCE_VIA, 9, "dereferenceVia");
known_value_constant!(ENTITY, 10, "entity");
known_value_constant!(HAS_NAME, 11, "hasName");
known_value_constant!(LANGUAGE, 12, "language");
known_value_constant!(ISSUER, 13, "issuer");
known_value_constant!(HOLDER, 14, "holder");
known_value_constant!(SALT, 15, "salt");
known_value_constant!(DATE, 16, "date");
known_value_constant!(UNKNOWN_VALUE, 17, "Unknown");
known_value_constant!(DIFF_EDITS, 20, "edits");

known_value_constant!(ATTACHMENT, 50, "attachment");
known_value_constant!(VENDOR, 51, "vendor");
known_value_constant!(CONFORMS_TO, 52, "conformsTo");

known_value_constant!(BODY, 100, "body");
known_value_constant!(RESULT, 101, "result");
known_value_constant!(ERROR, 102, "error");
known_value_constant!(OK_VALUE, 103, "OK");
known_value_constant!(PROCESSING_VALUE, 104, "Processing");
known_value_constant!(SENDER_PUBLIC_KEY, 105, "senderPublicKey");

known_value_constant!(SEED_TYPE, 200, "Seed");
known_value_constant!(PRIVATE_KEY_TYPE, 201, "PrivateKey");
known_value_constant!(PUBLIC_KEY_TYPE, 202, "PublicKey");
known_value_constant!(MASTER_KEY_TYPE, 203, "MasterKey");

known_value_constant!(ASSET, 300, "asset");
known_value_constant!(BITCOIN_VALUE, 301, "BTC");
known_value_constant!(ETHEREUM_VALUE, 302, "ETH");

known_value_constant!(NETWORK, 400, "network");
known_value_constant!(MAIN_NET_VALUE, 401, "MainNet");
known_value_constant!(TEST_NET_VALUE, 402, "TestNet");

known_value_constant!(BIP32_KEY_TYPE, 500, "BIP32Key");
known_value_constant!(CHAIN_CODE, 501, "chainCode");
known_value_constant!(DERIVATION_PATH_TYPE, 502, "DerivationPath");
known_value_constant!(PARENT_PATH, 503, "parent");
known_value_constant!(CHILDREN_PATH, 504, "children");
known_value_constant!(PARENT_FINGERPRINT, 505, "parentFingerprint");
known_value_constant!(PSBT_TYPE, 506, "PSBT");
known_value_constant!(OUTPUT_DESCRIPTOR_TYPE, 507, "OutputDescriptor");

#[doc(hidden)]
#[derive(Debug)]
pub struct LazyKnownValues {
    init: Once,
    data: Mutex<Option<KnownValuesStore>>,
}

impl LazyKnownValues {
    pub fn get(&self) -> std::sync::MutexGuard<'_, Option<KnownValuesStore>> {
        self.init.call_once(|| {
            let m = KnownValuesStore::new([
                IS_A,
                ID,
                VERIFIED_BY,
                NOTE,
                HAS_RECIPIENT,
                SSKR_SHARE,
                CONTROLLER,
                PUBLIC_KEYS,
                DEREFERENCE_VIA,
                ENTITY,
                HAS_NAME,
                LANGUAGE,
                ISSUER,
                HOLDER,
                SALT,
                DATE,
                UNKNOWN_VALUE,
                DIFF_EDITS,

                ATTACHMENT,
                VENDOR,
                CONFORMS_TO,

                BODY,
                RESULT,
                ERROR,
                OK_VALUE,
                PROCESSING_VALUE,
                SENDER_PUBLIC_KEY,

                SEED_TYPE,
                PRIVATE_KEY_TYPE,
                PUBLIC_KEY_TYPE,
                MASTER_KEY_TYPE,

                ASSET,
                BITCOIN_VALUE,
                ETHEREUM_VALUE,

                NETWORK,
                MAIN_NET_VALUE,
                TEST_NET_VALUE,

                BIP32_KEY_TYPE,
                CHAIN_CODE,
                DERIVATION_PATH_TYPE,
                PARENT_PATH,
                CHILDREN_PATH,
                PARENT_FINGERPRINT,
                PSBT_TYPE,
                OUTPUT_DESCRIPTOR_TYPE,
            ]);
            *self.data.lock().unwrap() = Some(m);
        });
        self.data.lock().unwrap()
    }
}

pub static KNOWN_VALUES: LazyKnownValues = LazyKnownValues {
    init: Once::new(),
    data: Mutex::new(None),
};

#[cfg(test)]
mod tests {
    use known_values::KNOWN_VALUES;

    use crate::extension::known_values;

    #[test]
    fn test_1() {
        assert_eq!(known_values::IS_A.value(), 1);
        assert_eq!(known_values::IS_A.name(), "isA");
        let binding = KNOWN_VALUES.get();
        let known_values = binding.as_ref().unwrap();
        assert_eq!(known_values.known_value_named("isA").unwrap().value(), 1);
    }
}
