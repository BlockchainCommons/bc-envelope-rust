use std::collections::HashMap;

use super::known_value::KnownValue;

/// A store that maps between Known Values and their assigned names.
///
/// The `KnownValuesStore` provides a bidirectional mapping between:
/// - Numeric values (u64) and their corresponding KnownValue instances
/// - String names and their corresponding KnownValue instances
///
/// This enables efficient lookup in both directions, making it possible to:
/// - Find the name for a given numeric value
/// - Find the numeric value for a given name
/// - Retrieve complete KnownValue instances by either name or value
///
/// The store is typically populated with predefined Known Values from the registry,
/// but can also be extended with custom values.
///
/// # Examples
///
/// ```
/// use bc_envelope::prelude::*;
/// use std::collections::HashMap;
///
/// // Create a store with predefined Known Values
/// let store = KnownValuesStore::new([
///     known_values::IS_A,
///     known_values::NOTE,
///     known_values::SIGNED
/// ]);
///
/// // Look up a Known Value by name
/// let is_a = store.known_value_named("isA").unwrap();
/// assert_eq!(is_a.value(), 1);
///
/// // Look up a name for a raw value
/// let name = store.name(KnownValue::new(3));
/// assert_eq!(name, "signed");
///
/// // Insert a custom Known Value
/// let mut custom_store = store.clone();
/// custom_store.insert(KnownValue::new_with_name(100u64, "customValue".to_string()));
/// assert_eq!(custom_store.known_value_named("customValue").unwrap().value(), 100);
/// ```
#[derive(Clone, Debug)]
pub struct KnownValuesStore {
    known_values_by_raw_value: HashMap<u64, KnownValue>,
    known_values_by_assigned_name: HashMap<String, KnownValue>,
}

impl KnownValuesStore {
    /// Creates a new KnownValuesStore with the provided Known Values.
    ///
    /// This constructor takes any iterable of KnownValue instances and populates
    /// the store with them, creating mappings from both raw values and names to 
    /// the corresponding KnownValue instances.
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Create a store with predefined Known Values
    /// let store = KnownValuesStore::new([
    ///     known_values::IS_A,
    ///     known_values::NOTE,
    ///     known_values::SIGNED
    /// ]);
    ///
    /// // Look up Known Values
    /// assert_eq!(store.known_value_named("isA").unwrap().value(), 1);
    /// assert_eq!(store.known_value_named("note").unwrap().value(), 4);
    /// ```
    pub fn new<T>(known_values: T) -> Self
    where
        T: IntoIterator<Item = KnownValue>,
    {
        let mut known_values_by_raw_value = HashMap::new();
        let mut known_values_by_assigned_name = HashMap::new();
        for known_value in known_values {
            Self::_insert(
                known_value,
                &mut known_values_by_raw_value,
                &mut known_values_by_assigned_name,
            );
        }
        Self {
            known_values_by_raw_value,
            known_values_by_assigned_name,
        }
    }

    /// Inserts a KnownValue into the store.
    ///
    /// If the KnownValue has an assigned name, it will be indexed by both its
    /// raw value and its name. If a KnownValue with the same raw value or name
    /// already exists in the store, it will be replaced.
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// let mut store = KnownValuesStore::default();
    /// store.insert(KnownValue::new_with_name(100u64, "customValue".to_string()));
    /// assert_eq!(store.known_value_named("customValue").unwrap().value(), 100);
    /// ```
    pub fn insert(&mut self, known_value: KnownValue) {
        Self::_insert(
            known_value,
            &mut self.known_values_by_raw_value,
            &mut self.known_values_by_assigned_name,
        );
    }

    /// Returns the assigned name for a KnownValue, if present in the store.
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// let store = KnownValuesStore::new([
    ///     known_values::IS_A,
    ///     known_values::NOTE
    /// ]);
    ///
    /// assert_eq!(store.assigned_name(&known_values::IS_A), Some("isA"));
    /// assert_eq!(store.assigned_name(&KnownValue::new(999)), None);
    /// ```
    pub fn assigned_name(&self, known_value: &KnownValue) -> Option<&str> {
        self.known_values_by_raw_value
            .get(&known_value.value())
            .and_then(|known_value| known_value.assigned_name())
    }

    /// Returns a human-readable name for a KnownValue.
    ///
    /// If the KnownValue has an assigned name in the store, that name is returned.
    /// Otherwise, the KnownValue's default name (which may be its numeric value as a string)
    /// is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// let store = KnownValuesStore::new([
    ///     known_values::IS_A,
    ///     known_values::NOTE
    /// ]);
    ///
    /// assert_eq!(store.name(known_values::IS_A), "isA");
    /// assert_eq!(store.name(KnownValue::new(999)), "999");
    /// ```
    pub fn name(&self, known_value: KnownValue) -> String {
        self.assigned_name(&known_value)
            .map(|name| name.to_string())
            .unwrap_or_else(|| known_value.name())
    }

    /// Looks up a KnownValue by its assigned name.
    ///
    /// Returns a reference to the KnownValue if found, or None if no KnownValue
    /// with the given name exists in the store.
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// let store = KnownValuesStore::new([
    ///     known_values::IS_A,
    ///     known_values::NOTE
    /// ]);
    ///
    /// let is_a = store.known_value_named("isA").unwrap();
    /// assert_eq!(is_a.value(), 1);
    ///
    /// assert!(store.known_value_named("nonexistent").is_none());
    /// ```
    pub fn known_value_named(&self, assigned_name: &str) -> Option<&KnownValue> {
        self.known_values_by_assigned_name.get(assigned_name)
    }

    /// Retrieves a KnownValue for a raw value, using a store if provided.
    ///
    /// This static method allows looking up a KnownValue by its raw numeric value:
    /// - If a store is provided and contains a mapping for the raw value, that KnownValue is returned
    /// - Otherwise, a new KnownValue with no assigned name is created and returned
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// let store = KnownValuesStore::new([
    ///     known_values::IS_A,
    ///     known_values::NOTE
    /// ]);
    ///
    /// // Known value from store
    /// let is_a = KnownValuesStore::known_value_for_raw_value(1, Some(&store));
    /// assert_eq!(is_a.name(), "isA");
    ///
    /// // Unknown value creates a new KnownValue
    /// let unknown = KnownValuesStore::known_value_for_raw_value(999, Some(&store));
    /// assert_eq!(unknown.name(), "999");
    ///
    /// // No store provided also creates a new KnownValue
    /// let unknown = KnownValuesStore::known_value_for_raw_value(1, None);
    /// assert_eq!(unknown.name(), "1");
    /// ```
    pub fn known_value_for_raw_value(raw_value: u64, known_values: Option<&Self>) -> KnownValue {
        known_values
            .and_then(|known_values| known_values.known_values_by_raw_value.get(&raw_value))
            .cloned()
            .unwrap_or_else(|| KnownValue::new(raw_value))
    }

    /// Attempts to find a KnownValue by its name, using a store if provided.
    ///
    /// This static method allows looking up a KnownValue by its name:
    /// - If a store is provided and contains a mapping for the name, that KnownValue is returned
    /// - Otherwise, None is returned
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// let store = KnownValuesStore::new([
    ///     known_values::IS_A,
    ///     known_values::NOTE
    /// ]);
    ///
    /// // Known value from store
    /// let is_a = KnownValuesStore::known_value_for_name("isA", Some(&store));
    /// assert_eq!(is_a.unwrap().value(), 1);
    ///
    /// // Unknown name returns None
    /// assert!(KnownValuesStore::known_value_for_name("unknown", Some(&store)).is_none());
    ///
    /// // No store provided also returns None
    /// assert!(KnownValuesStore::known_value_for_name("isA", None).is_none());
    /// ```
    pub fn known_value_for_name(name: &str, known_values: Option<&Self>) -> Option<KnownValue> {
        known_values
            .and_then(|known_values| known_values.known_value_named(name))
            .cloned()
    }

    /// Returns a human-readable name for a KnownValue, using a store if provided.
    ///
    /// This static method allows getting a name for a KnownValue:
    /// - If a store is provided and contains a mapping for the KnownValue, its assigned name is returned
    /// - Otherwise, the KnownValue's default name (which may be its numeric value as a string) is returned
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// let store = KnownValuesStore::new([
    ///     known_values::IS_A,
    ///     known_values::NOTE
    /// ]);
    ///
    /// // Known value from store
    /// let name = KnownValuesStore::name_for_known_value(known_values::IS_A, Some(&store));
    /// assert_eq!(name, "isA");
    ///
    /// // Unknown value in store uses KnownValue's name method
    /// let name = KnownValuesStore::name_for_known_value(KnownValue::new(999), Some(&store));
    /// assert_eq!(name, "999");
    ///
    /// // No store provided also uses KnownValue's name method
    /// let name = KnownValuesStore::name_for_known_value(known_values::IS_A, None);
    /// assert_eq!(name, "isA");
    /// ```
    pub fn name_for_known_value(known_value: KnownValue, known_values: Option<&Self>) -> String {
        known_values
            .and_then(|known_values| known_values.assigned_name(&known_value))
            .map(|assigned_name| assigned_name.to_string())
            .unwrap_or_else(|| known_value.name())
    }

    /// Internal helper method to insert a KnownValue into the store's maps.
    fn _insert(
        known_value: KnownValue,
        known_values_by_raw_value: &mut HashMap<u64, KnownValue>,
        known_values_by_assigned_name: &mut HashMap<String, KnownValue>,
    ) {
        known_values_by_raw_value.insert(known_value.value(), known_value.clone());
        if let Some(name) = known_value.assigned_name() {
            known_values_by_assigned_name.insert(name.to_string(), known_value);
        }
    }
}

/// Default implementation creates an empty KnownValuesStore.
impl Default for KnownValuesStore {
    fn default() -> Self {
        Self::new([])
    }
}
