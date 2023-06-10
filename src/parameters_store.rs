use std::collections::HashMap;

use crate::Parameter;

 #[derive(Clone, Debug)]
 pub struct ParametersStore {
     dict: HashMap<Parameter, String>,
 }

 impl ParametersStore {
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

    pub fn insert(&mut self, parameter: Parameter) {
        Self::_insert(parameter, &mut self.dict);
    }

    pub fn assigned_name(&self, parameter: &Parameter) -> Option<&str> {
        self.dict.get(parameter).map(|name| name.as_str())
    }

    pub fn name(&self, parameter: &Parameter) -> String {
        self.assigned_name(parameter)
            .map(|name| name.to_string())
            .unwrap_or_else(|| parameter.name())
    }

    pub fn name_for_parameter(parameter: &Parameter, parameters: Option<&Self>) -> String {
        parameters
            .and_then(|parameters| parameters.assigned_name(parameter))
            .map(|name| name.to_string())
            .unwrap_or_else(|| parameter.name())
    }

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

impl Default for ParametersStore {
    fn default() -> Self {
        Self::new([])
    }
}
