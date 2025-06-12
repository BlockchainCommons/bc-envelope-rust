use std::sync::{Mutex, Once};

use paste::paste;

use super::{Parameter, ParametersStore};

/// A macro that declares a parameter at compile time.
///
/// This macro generates a constant `Parameter` with a given numeric value and
/// name. It also creates a companion constant for the numeric value with a
/// suffix `_VALUE`.
///
/// # Examples
///
/// ```ignore
/// use bc_envelope::prelude::*;
/// use bc_envelope::parameter_constant;
/// use bc_envelope::extension::expressions::Parameter;
///
/// // Define a custom parameter
/// parameter_constant!(MY_PARAM, 100, "myParam");
///
/// // Usage
/// assert_eq!(MY_PARAM_VALUE, 100);
/// assert_eq!(MY_PARAM.name(), "myParam");
/// ```
#[macro_export]
macro_rules! parameter_constant {
    ($const_name:ident, $value:expr, $name:expr) => {
        paste! {
            pub const [<$const_name _VALUE>]: u64 = $value;
        }
        pub const $const_name: Parameter =
            Parameter::new_with_static_name($value, $name);
    };
}

// The blank parameter, used for single-parameter functions.
//
// This parameter is commonly used when a function needs only one parameter
// and the parameter's purpose is clear from context. It is denoted as `❰_❱`
// in envelope notation.
parameter_constant!(BLANK, 1, "_");

// The left-hand side parameter, used for binary operations.
//
// This parameter typically represents the first operand in a binary operation
// such as addition or multiplication.
parameter_constant!(LHS, 2, "lhs");

// The right-hand side parameter, used for binary operations.
//
// This parameter typically represents the second operand in a binary operation
// such as addition or multiplication.
parameter_constant!(RHS, 3, "rhs");

/// A helper type for lazy initialization of the global parameters store.
///
/// This is an implementation detail that handles thread-safe, one-time
/// initialization of the global parameters registry.
#[doc(hidden)]
#[derive(Debug)]
pub struct LazyParameters {
    init: Once,
    data: Mutex<Option<ParametersStore>>,
}

impl LazyParameters {
    /// Gets a reference to the global parameters store, initializing it if
    /// necessary.
    ///
    /// This method ensures the global parameters store is initialized only
    /// once, in a thread-safe manner.
    ///
    /// # Returns
    ///
    /// A mutex guard containing a reference to the parameters store
    pub fn get(&self) -> std::sync::MutexGuard<'_, Option<ParametersStore>> {
        self.init.call_once(|| {
            let m = ParametersStore::new([BLANK, LHS, RHS]);
            *self.data.lock().unwrap() = Some(m);
        });
        self.data.lock().unwrap()
    }
}

/// The global shared store of known parameters.
///
/// This provides access to a global registry of standard parameters used
/// in envelope expressions. It is lazily initialized the first time it's
/// accessed.
///
/// # Examples
///
/// ```
/// use bc_envelope::{
///     extension::expressions::{GLOBAL_PARAMETERS, parameters},
///     prelude::*,
/// };
///
/// // Access the global parameters store
/// let parameters_store = GLOBAL_PARAMETERS.get();
/// if let Some(store) = &*parameters_store {
///     // Use the store to look up parameter names
///     assert_eq!(store.name(&parameters::LHS), "lhs");
/// }
/// ```
pub static GLOBAL_PARAMETERS: LazyParameters =
    LazyParameters { init: Once::new(), data: Mutex::new(None) };
