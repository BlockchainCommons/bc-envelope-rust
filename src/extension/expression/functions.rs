use std::sync::{Once, Mutex};
use paste::paste;

use super::{Function, FunctionsStore};

/// A macro that declares a function at compile time.
#[macro_export]
macro_rules! function_constant {
    ($const_name:ident, $value:expr, $name:expr) => {
        paste! {
            pub const [<$const_name _VALUE>]: u64 = $value;
        }
        pub const $const_name: Function = Function::new_with_static_name($value, $name);
    };
}

function_constant!(ADD, 1, "add"); // addition
function_constant!(SUB, 2, "sub"); // subtraction
function_constant!(MUL, 3, "mul"); // multiplication
function_constant!(DIV, 4, "div"); // division
function_constant!(NEG, 5, "neg"); // unary negation
function_constant!(LT, 6, "lt"); // less than
function_constant!(LE, 7, "le"); // less than or equal to
function_constant!(GT, 8, "gt"); // greater than
function_constant!(GE, 9, "ge"); // greater than or equal to
function_constant!(EQ, 10, "eq"); // equal to
function_constant!(NE, 11, "ne"); // not equal to
function_constant!(AND, 12, "and"); // logical and
function_constant!(OR, 13, "or"); // logical or
function_constant!(XOR, 14, "xor"); // logical exclusive or
function_constant!(NOT, 15, "not"); // logical not

#[doc(hidden)]
pub struct LazyFunctions {
    init: Once,
    data: Mutex<Option<FunctionsStore>>,
}

impl LazyFunctions {
    pub fn get(&self) -> std::sync::MutexGuard<'_, Option<FunctionsStore>> {
        self.init.call_once(|| {
            let m = FunctionsStore::new([
                ADD,
                SUB,
                MUL,
                DIV,
            ]);
            *self.data.lock().unwrap() = Some(m);
        });
        self.data.lock().unwrap()
    }
}

/// The global shared store of known functions.
pub static GLOBAL_FUNCTIONS: LazyFunctions = LazyFunctions {
    init: Once::new(),
    data: Mutex::new(None),
};
