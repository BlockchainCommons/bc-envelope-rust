use std::collections::HashMap;

use super::Function;

/// A store that maps functions to their assigned names.
///
/// `FunctionsStore` maintains a registry of functions and their human-readable
/// names, which is useful for displaying and debugging expression functions.
/// The store allows for consistent name resolution of functions used in
/// expressions.
///
/// # Examples
///
/// ```
/// use bc_envelope::{
///     extension::expressions::{FunctionsStore, functions},
///     prelude::*,
/// };
///
/// // Create a store with some known functions
/// let store =
///     FunctionsStore::new([functions::ADD, functions::SUB, functions::MUL]);
///
/// // Look up the name of a function
/// assert_eq!(store.name(&functions::ADD), "add");
/// ```
#[derive(Clone, Debug)]
pub struct FunctionsStore {
    dict: HashMap<Function, String>,
}

impl FunctionsStore {
    /// Creates a new `FunctionsStore` with the given functions.
    ///
    /// # Parameters
    ///
    /// * `functions` - An iterable of `Function` instances to store
    ///
    /// # Returns
    ///
    /// A new `FunctionsStore` containing the functions
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::{
    ///     extension::expressions::{FunctionsStore, functions},
    ///     prelude::*,
    /// };
    ///
    /// // Create a store with standard arithmetic functions
    /// let store = FunctionsStore::new([
    ///     functions::ADD,
    ///     functions::SUB,
    ///     functions::MUL,
    ///     functions::DIV,
    /// ]);
    /// ```
    pub fn new<T>(functions: T) -> Self
    where
        T: IntoIterator<Item = Function>,
    {
        let mut dict = HashMap::new();
        for function in functions {
            Self::_insert(function, &mut dict);
        }
        Self { dict }
    }

    /// Inserts a function into the store.
    ///
    /// # Parameters
    ///
    /// * `function` - The function to insert
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::{
    ///     extension::expressions::{Function, FunctionsStore, functions},
    ///     prelude::*,
    /// };
    ///
    /// let mut store = FunctionsStore::default();
    /// store.insert(functions::ADD);
    /// store.insert(Function::new_with_static_name(100, "myCustomFunction"));
    /// ```
    pub fn insert(&mut self, function: Function) {
        Self::_insert(function, &mut self.dict);
    }

    /// Returns the assigned name for a function, if it exists in the store.
    ///
    /// # Parameters
    ///
    /// * `function` - The function to look up
    ///
    /// # Returns
    ///
    /// Some string slice with the function name if found, or None if not found
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::{
    ///     extension::expressions::{FunctionsStore, functions},
    ///     prelude::*,
    /// };
    ///
    /// let store = FunctionsStore::new([functions::ADD]);
    /// assert_eq!(store.assigned_name(&functions::ADD), Some("add"));
    /// assert_eq!(store.assigned_name(&functions::SUB), None);
    /// ```
    pub fn assigned_name(&self, function: &Function) -> Option<&str> {
        self.dict.get(function).map(|name| name.as_str())
    }

    /// Returns the name for a function, either from this store or from the
    /// function itself.
    ///
    /// If the function exists in the store, returns its assigned name.
    /// Otherwise, returns the function's own name.
    ///
    /// # Parameters
    ///
    /// * `function` - The function to look up
    ///
    /// # Returns
    ///
    /// The name of the function as a String
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::{
    ///     extension::expressions::{FunctionsStore, functions},
    ///     prelude::*,
    /// };
    ///
    /// let store = FunctionsStore::new([functions::ADD]);
    /// assert_eq!(store.name(&functions::ADD), "add");
    /// // Not in store, so uses function's own name
    /// assert_eq!(store.name(&functions::SUB), "sub");
    /// ```
    pub fn name(&self, function: &Function) -> String {
        self.assigned_name(function)
            .map(|name| name.to_string())
            .unwrap_or_else(|| function.name())
    }

    /// A static method that returns the name of a function, using an optional
    /// store.
    ///
    /// This utility method is useful when you have an optional store and want
    /// to get a function name without additional unwrapping logic.
    ///
    /// # Parameters
    ///
    /// * `function` - The function to look up
    /// * `functions` - An optional reference to a FunctionsStore
    ///
    /// # Returns
    ///
    /// The name of the function as a String
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::{
    ///     extension::expressions::{FunctionsStore, functions},
    ///     prelude::*,
    /// };
    ///
    /// let store = FunctionsStore::new([functions::ADD]);
    ///
    /// // Using the store
    /// assert_eq!(
    ///     FunctionsStore::name_for_function(&functions::ADD, Some(&store)),
    ///     "add"
    /// );
    ///
    /// // Without a store
    /// assert_eq!(
    ///     FunctionsStore::name_for_function(&functions::ADD, None),
    ///     "add"
    /// );
    /// ```
    pub fn name_for_function(
        function: &Function,
        functions: Option<&Self>,
    ) -> String {
        functions
            .and_then(|functions| functions.assigned_name(function))
            .map(|name| name.to_string())
            .unwrap_or_else(|| function.name())
    }

    /// Private helper method to insert a function into the dictionary.
    ///
    /// This handles the validation and naming logic for function insertion.
    fn _insert(function: Function, dict: &mut HashMap<Function, String>) {
        match function {
            Function::Known(_, _) => {
                let name = function.name();
                dict.insert(function, name);
            }
            _ => panic!(),
        }
    }
}

/// Provides a default empty store.
impl Default for FunctionsStore {
    fn default() -> Self { Self::new([]) }
}
