use std::collections::HashMap;

use super::Function;

/// A type that maps functions to their assigned names.
#[derive(Clone, Debug)]
pub struct FunctionsStore {
    dict: HashMap<Function, String>,
}

impl FunctionsStore {
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

    pub fn insert(&mut self, function: Function) {
        Self::_insert(function, &mut self.dict);
    }

    pub fn assigned_name(&self, function: &Function) -> Option<&str> {
        self.dict.get(function).map(|name| name.as_str())
    }

    pub fn name(&self, function: &Function) -> String {
        self.assigned_name(function)
            .map(|name| name.to_string())
            .unwrap_or_else(|| function.name())
    }

    pub fn name_for_function(function: &Function, functions: Option<&Self>) -> String {
        functions
            .and_then(|functions| functions.assigned_name(function))
            .map(|name| name.to_string())
            .unwrap_or_else(|| function.name())
    }

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

impl Default for FunctionsStore {
    fn default() -> Self {
        Self::new([])
    }
}
