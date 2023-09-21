pub mod known_value;
pub use known_value::KnownValue;

mod known_values_store;
pub use known_values_store::KnownValuesStore;

use std::sync::{Once, Mutex};
use paste::paste;


/// A macro that declares a known value at compile time.
#[macro_export]
macro_rules! known_value_constant {
    ($const_name:ident, $value:expr, $name:expr) => {
        paste! {
            pub const [<$const_name _RAW>]: u64 = $value;
        }
        pub const $const_name: KnownValue = KnownValue::new_with_static_name($value, $name);
    };
}

// ```swift
// public extension KnownValue {
//     static let isA = KnownValue(1, "isA")
//     static let id = KnownValue(2, "id")
//     static let verifiedBy = KnownValue(3, "verifiedBy")
//     static let note = KnownValue(4, "note")
//     static let hasRecipient = KnownValue(5, "hasRecipient")
//     static let sskrShare = KnownValue(6, "sskrShare")
//     static let controller = KnownValue(7, "controller")
//     static let publicKeys = KnownValue(8, "publicKeys")
//     static let dereferenceVia = KnownValue(9, "dereferenceVia")
//     static let entity = KnownValue(10, "entity")
//     static let hasName = KnownValue(11, "hasName")
//     static let language = KnownValue(12, "language")
//     static let issuer = KnownValue(13, "issuer")
//     static let holder = KnownValue(14, "holder")
//     static let salt = KnownValue(15, "salt")
//     static let date = KnownValue(16, "date")
//     static let unknown = KnownValue(17, "Unknown")
//     static let diffEdits = KnownValue(20, "edits")
//     static let attachment = KnownValue(50, "attachment")
//     static let vendor = KnownValue(51, "vendor")
//     static let conformsTo = KnownValue(52, "conformsTo")
//     static let body = KnownValue(100, "body")
//     static let result = KnownValue(101, "result")
//     static let error = KnownValue(102, "error")
//     static let OK = KnownValue(103, "OK")
//     static let Processing = KnownValue(104, "Processing")
//     static let Seed = KnownValue(200, "Seed")
//     static let PrivateKey = KnownValue(201, "PrivateKey")
//     static let PublicKey = KnownValue(202, "PublicKey")
//     static let MasterKey = KnownValue(203, "MasterKey")
//     static let asset = KnownValue(300, "asset")
//     static let Bitcoin = KnownValue(301, "BTC")
//     static let Ethereum = KnownValue(302, "ETH")
//     static let network = KnownValue(400, "network")
//     static let MainNet = KnownValue(401, "MainNet")
//     static let TestNet = KnownValue(402, "TestNet")
//     static let BIP32Key = KnownValue(500, "BIP32Key")
//     static let chainCode = KnownValue(501, "chainCode")
//     static let DerivationPath = KnownValue(502, "DerivationPath")
//     static let parentPath = KnownValue(503, "parent")
//     static let childrenPath = KnownValue(504, "children")
//     static let parentFingerprint = KnownValue(505, "parentFingerprint")
//     static let PSBT = KnownValue(506, "PSBT")
//     static let OutputDescriptor = KnownValue(507, "OutputDescriptor")
//     static let outputDescriptor = KnownValue(508, "outputDescriptor")
// }
// ```

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
    use crate::known_values::{self, KNOWN_VALUES};

    #[test]
    fn test_1() {
        assert_eq!(known_values::IS_A.value(), 1);
        assert_eq!(known_values::IS_A.name(), "isA");
        let binding = KNOWN_VALUES.get();
        let known_values = binding.as_ref().unwrap();
        assert_eq!(known_values.known_value_named("isA").unwrap().value(), 1);
    }
}
