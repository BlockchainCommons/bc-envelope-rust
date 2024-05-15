use std::sync::{Once, Mutex};
use paste::paste;

use super::{Parameter, ParametersStore};

/// A macro that declares a parameter at compile time.
#[macro_export]
macro_rules! parameter_constant {
    ($const_name:ident, $value:expr, $name:expr) => {
        paste! {
            pub const [<$const_name _VALUE>]: u64 = $value;
        }
        pub const $const_name: Parameter = Parameter::new_with_static_name($value, $name);
    };
}

parameter_constant!(BLANK, 1, "_");
parameter_constant!(LHS, 2, "lhs");
parameter_constant!(RHS, 3, "rhs");

#[doc(hidden)]
#[derive(Debug)]
pub struct LazyParameters {
    init: Once,
    data: Mutex<Option<ParametersStore>>,
}

impl LazyParameters {
    pub fn get(&self) -> std::sync::MutexGuard<'_, Option<ParametersStore>> {
        self.init.call_once(|| {
            let m = ParametersStore::new([
                BLANK,
                LHS,
                RHS,
            ]);
            *self.data.lock().unwrap() = Some(m);
        });
        self.data.lock().unwrap()
    }
}

/// The global shared store of known parameters.
pub static GLOBAL_PARAMETERS: LazyParameters = LazyParameters {
    init: Once::new(),
    data: Mutex::new(None),
};
