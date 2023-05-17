use std::sync::{Once, Mutex};
use crate::{Parameter, KnownParameters};
use paste::paste;

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

pub struct LazyParameters {
    init: Once,
    data: Mutex<Option<KnownParameters>>,
}

impl LazyParameters {
    pub fn get(&self) -> std::sync::MutexGuard<'_, Option<KnownParameters>> {
        self.init.call_once(|| {
            let m = KnownParameters::new([
                BLANK,
                LHS,
                RHS,
            ]);
            *self.data.lock().unwrap() = Some(m);
        });
        self.data.lock().unwrap()
    }
}

pub static PARAMETERS: LazyParameters = LazyParameters {
    init: Once::new(),
    data: Mutex::new(None),
};
