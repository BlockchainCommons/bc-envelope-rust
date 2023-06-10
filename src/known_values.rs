use std::sync::{Once, Mutex};
use crate::{KnownValue, KnownValuesStore};
use paste::paste;

#[macro_export]
macro_rules! known_value_constant {
    ($const_name:ident, $value:expr, $name:expr) => {
        paste! {
            pub const [<$const_name _VALUE>]: u64 = $value;
        }
        pub const $const_name: KnownValue = KnownValue::new_with_static_name($value, $name);
    };
}

// Predicate declaring the subject is known by the identifier object.
known_value_constant!(ID, 1, "id");

// Predicate declaring the subject is of a type identified by the object.
known_value_constant!(IS_A, 2, "isA");

// Predicate declaring the subject is signed by the `Signature` object.
known_value_constant!(VERIFIED_BY, 3, "verifiedBy");

// Predicate declaring the subject is accompanied by a human-readable note object.
known_value_constant!(NOTE, 4, "note");

// Predicate declaring the subject can be decrypted by the ephemeral key contained
// in the `SealedMessage` object.
known_value_constant!(HAS_RECIPIENT, 5, "hasRecipient");

// Predicate declaring the subject can be decryped by a quorum of `SSKRShare`s
// including the one in the object.
known_value_constant!(SSKR_SHARE, 6, "sskrShare");

// Predicate declaring that the document is controlled by the party identified by
// the object.
known_value_constant!(CONTROLLER, 7, "controller");

// Predicate declaring that the party identified by the subject holds the private keys
// to the `PublicKeyBase` object.
known_value_constant!(PUBLIC_KEYS, 8, "publicKeys");

// Predicate declaring that the content referenced by the subject can be
// dereferenced using the information in the object.
known_value_constant!(DEREFERENCE_VIA, 9, "dereferenceVia");

// Predicate declaring that the entity referenced by the subject is specified in
// the object.
known_value_constant!(ENTITY, 10, "entity");

// Predicate declaring that the entity referenced by the subject is known by the
// name in the object.
known_value_constant!(HAS_NAME, 11, "hasName");

// Predicate declaring the the subject `String` is written in the language of the
// ISO language code object.
known_value_constant!(LANGUAGE, 12, "language");

// Predicate declaring that the issuer of the object referenced in the subject is
// the entity referenced in the object.
known_value_constant!(ISSUER, 13, "issuer");

// Predicate declaring that the holder of the credential or certificate referenced
// in the subject is the entity referenced in the object.
known_value_constant!(HOLDER, 14, "holder");

// Predicate declaring that the object is random salt used to decorrelate the
// digest of the subject.
known_value_constant!(SALT, 15, "salt");

// Predicate declaring a primary datestamp on the envelope.
known_value_constant!(DATE, 16, "date");


// Predicate declaring that the object is a set of edits using by the
// `Envelope.transform(edits:)` method to transform a `source` envelope into a `target`
// envelope.
known_value_constant!(DIFF_EDITS, 20, "edits");


// Predicate declaring that the object is the body (parameters of) a distributed
// request identified by the subject.
known_value_constant!(BODY, 100, "body");

// Predicate declaring that the object is the success result of the request
// identified by the subject.
known_value_constant!(RESULT, 101, "result");

// Predicate declaring that the object is the failure result of the request
// identified by the subject.
known_value_constant!(ERROR, 102, "error");

// Object providing the success result of a request that has no other return value.
known_value_constant!(OK, 103, "ok");

// Object providing the "in processing" result of a request.
known_value_constant!(PROCESSING, 104, "processing");

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
    use crate::known_value::KNOWN_VALUES;

    #[test]
    fn test_1() {
        use crate::*;
        assert_eq!(known_value::IS_A.value(), 2);
        assert_eq!(known_value::IS_A.name(), Some("isA").unwrap());
        let binding = KNOWN_VALUES.get();
        let known_values = binding.as_ref().unwrap();
        assert_eq!(known_values.known_value_named("isA").unwrap().value(), 2);
    }
}
