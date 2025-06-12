use std::collections::HashMap;

use super::Parameter;

/// A store that maps parameters to their assigned names.
///
/// `ParametersStore` maintains a registry of parameters and their
/// human-readable names, which is useful for displaying and debugging
/// expression parameters. The store allows for consistent name resolution of
/// parameters used in expressions.
///
/// # Examples
///
/// ```
/// use bc_envelope::{
///     extension::expressions::{ParametersStore, parameters},
///     prelude::*,
/// };
///
/// // Create a store with common parameters
/// let store = ParametersStore::new([
///     parameters::LHS,
///     parameters::RHS,
///     parameters::BLANK,
/// ]);
///
/// // Look up the name of a parameter
/// assert_eq!(store.name(&parameters::LHS), "lhs");
/// ```
#[derive(Clone, Debug)]
pub struct ParametersStore {
    dict: HashMap<Parameter, String>,
}

impl ParametersStore {
    /// Creates a new `ParametersStore` with the given parameters.
    ///
    /// # Parameters
    ///
    /// * `parameters` - An iterable of `Parameter` instances to store
    ///
    /// # Returns
    ///
    /// A new `ParametersStore` containing the parameters
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::{
    ///     extension::expressions::{ParametersStore, parameters},
    ///     prelude::*,
    /// };
    ///
    /// // Create a store with standard parameters
    /// let store = ParametersStore::new([
    ///     parameters::LHS,
    ///     parameters::RHS,
    ///     parameters::BLANK,
    /// ]);
    /// ```
    pub fn new<T>(parameters: T) -> Self
    where
        T: IntoIterator<Item = Parameter>,
    {
        let mut dict = HashMap::new();
        for parameter in parameters {
            Self::_insert(parameter, &mut dict);
        }
        Self { dict }
    }

    /// Inserts a parameter into the store.
    ///
    /// # Parameters
    ///
    /// * `parameter` - The parameter to insert
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::{
    ///     extension::expressions::{Parameter, ParametersStore, parameters},
    ///     prelude::*,
    /// };
    ///
    /// let mut store = ParametersStore::default();
    /// store.insert(parameters::LHS);
    /// store.insert(Parameter::new_with_static_name(100, "myCustomParameter"));
    /// ```
    pub fn insert(&mut self, parameter: Parameter) {
        Self::_insert(parameter, &mut self.dict);
    }

    /// Returns the assigned name for a parameter, if it exists in the store.
    ///
    /// # Parameters
    ///
    /// * `parameter` - The parameter to look up
    ///
    /// # Returns
    ///
    /// Some string slice with the parameter name if found, or None if not found
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::{
    ///     extension::expressions::{ParametersStore, parameters},
    ///     prelude::*,
    /// };
    ///
    /// let store = ParametersStore::new([parameters::LHS]);
    /// assert_eq!(store.assigned_name(&parameters::LHS), Some("lhs"));
    /// assert_eq!(store.assigned_name(&parameters::RHS), None);
    /// ```
    pub fn assigned_name(&self, parameter: &Parameter) -> Option<&str> {
        self.dict.get(parameter).map(|name| name.as_str())
    }

    /// Returns the name for a parameter, either from this store or from the
    /// parameter itself.
    ///
    /// If the parameter exists in the store, returns its assigned name.
    /// Otherwise, returns the parameter's own name.
    ///
    /// # Parameters
    ///
    /// * `parameter` - The parameter to look up
    ///
    /// # Returns
    ///
    /// The name of the parameter as a String
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::{
    ///     extension::expressions::{ParametersStore, parameters},
    ///     prelude::*,
    /// };
    ///
    /// let store = ParametersStore::new([parameters::LHS]);
    /// assert_eq!(store.name(&parameters::LHS), "lhs");
    /// // Not in store, so uses parameter's own name
    /// assert_eq!(store.name(&parameters::RHS), "rhs");
    /// ```
    pub fn name(&self, parameter: &Parameter) -> String {
        self.assigned_name(parameter)
            .map(|name| name.to_string())
            .unwrap_or_else(|| parameter.name())
    }

    /// A static method that returns the name of a parameter, using an optional
    /// store.
    ///
    /// This utility method is useful when you have an optional store and want
    /// to get a parameter name without additional unwrapping logic.
    ///
    /// # Parameters
    ///
    /// * `parameter` - The parameter to look up
    /// * `parameters` - An optional reference to a ParametersStore
    ///
    /// # Returns
    ///
    /// The name of the parameter as a String
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::{
    ///     extension::expressions::{ParametersStore, parameters},
    ///     prelude::*,
    /// };
    ///
    /// let store = ParametersStore::new([parameters::LHS]);
    ///
    /// // Using the store
    /// assert_eq!(
    ///     ParametersStore::name_for_parameter(&parameters::LHS, Some(&store)),
    ///     "lhs"
    /// );
    ///
    /// // Without a store
    /// assert_eq!(
    ///     ParametersStore::name_for_parameter(&parameters::LHS, None),
    ///     "lhs"
    /// );
    /// ```
    pub fn name_for_parameter(
        parameter: &Parameter,
        parameters: Option<&Self>,
    ) -> String {
        parameters
            .and_then(|parameters| parameters.assigned_name(parameter))
            .map(|name| name.to_string())
            .unwrap_or_else(|| parameter.name())
    }

    /// Private helper method to insert a parameter into the dictionary.
    ///
    /// This handles the validation and naming logic for parameter insertion.
    fn _insert(parameter: Parameter, dict: &mut HashMap<Parameter, String>) {
        match parameter {
            Parameter::Known(_, _) => {
                let name = parameter.name();
                dict.insert(parameter, name);
            }
            _ => panic!(),
        }
    }
}

/// Provides a default empty store.
impl Default for ParametersStore {
    fn default() -> Self { Self::new([]) }
}
