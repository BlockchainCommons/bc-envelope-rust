use std::collections::HashMap;

use crate::KnownValue;

#[derive(Clone, Debug)]
pub struct KnownValuesStore {
    known_values_by_raw_value: HashMap<u64, KnownValue>,
    known_values_by_assigned_name: HashMap<String, KnownValue>,
}

impl KnownValuesStore {
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

    pub fn insert(&mut self, known_value: KnownValue) {
        Self::_insert(
            known_value,
            &mut self.known_values_by_raw_value,
            &mut self.known_values_by_assigned_name,
        );
    }

    pub fn assigned_name(&self, known_value: &KnownValue) -> Option<&str> {
        self.known_values_by_raw_value
            .get(&known_value.value())
            .and_then(|known_value| known_value.assigned_name())
    }

    pub fn name(&self, known_value: KnownValue) -> String {
        self.assigned_name(&known_value)
            .map(|name| name.to_string())
            .unwrap_or_else(|| known_value.name())
    }

    pub fn known_value_named(&self, assigned_name: &str) -> Option<&KnownValue> {
        self.known_values_by_assigned_name.get(assigned_name)
    }

    pub fn known_value_for_raw_value(raw_value: u64, known_values: Option<&Self>) -> KnownValue {
        known_values
            .and_then(|known_values| known_values.known_values_by_raw_value.get(&raw_value))
            .cloned()
            .unwrap_or_else(|| KnownValue::new(raw_value))
    }

    pub fn known_value_for_name(name: &str, known_values: Option<&Self>) -> Option<KnownValue> {
        known_values
            .and_then(|known_values| known_values.known_value_named(name))
            .cloned()
    }

    pub fn name_for_known_value(known_value: KnownValue, known_values: Option<&Self>) -> String {
        known_values
            .and_then(|known_values| known_values.assigned_name(&known_value))
            .map(|assigned_name| assigned_name.to_string())
            .unwrap_or_else(|| known_value.name())
    }

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

impl Default for KnownValuesStore {
    fn default() -> Self {
        Self::new([])
    }
}
