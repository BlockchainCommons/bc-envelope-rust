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
            pub const [<$const_name _VALUE>]: u64 = $value;
        }
        pub const $const_name: KnownValue = KnownValue::new_with_static_name($value, $name);
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
known_value_constant!(UNKNOWN, 17, "Unknown");
known_value_constant!(DIFF_EDITS, 20, "edits");
known_value_constant!(BODY, 100, "body");
known_value_constant!(RESULT, 101, "result");
known_value_constant!(ERROR, 102, "error");
known_value_constant!(OK, 103, "OK");
known_value_constant!(PROCESSING, 104, "Processing");

#[doc(hidden)]
pub struct LazyKnownValues {
    init: Once,
    data: Mutex<Option<KnownValuesStore>>,
}

impl LazyKnownValues {
    pub fn get(&self) -> std::sync::MutexGuard<'_, Option<KnownValuesStore>> {
        self.init.call_once(|| {
            let m = KnownValuesStore::new([
                ID,
                IS_A,
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
                DIFF_EDITS,
                BODY,
                RESULT,
                ERROR,
                OK,
                PROCESSING,
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
