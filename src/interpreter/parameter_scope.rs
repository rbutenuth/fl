use std::collections::HashMap;
use std::sync::Arc;

use crate::interpreter::scope::{MapScope, ScopeEntry};
use crate::{FlError, Value, interpreter::Scope};

pub struct ParameterScope {
    map_scope: MapScope,
    name_to_index: Arc<HashMap<String, usize>>,
    parameters: Vec<Value>,
}

impl ParameterScope {
    pub fn new(
        name: impl Into<String>,
        next: Option<Arc<dyn Scope>>,
        name_to_index: Arc<HashMap<String, usize>>,
        parameters: Vec<Value>
    ) -> Self {
        ParameterScope {
            map_scope: MapScope::new(name, next),
            name_to_index,
            parameters,
        }
    }
}

impl Scope for ParameterScope {
    fn get_next(&self) -> Option<Arc<dyn Scope>> {
        self.map_scope.get_next()
    }

    fn get_name(&self) -> String {
        self.map_scope.get_name()
    }

    fn get(&self, key: &str) -> Value {
        match self.name_to_index.get(key) {
            Some(&index) => self.parameters[index].clone(),
            None => self.map_scope.get(key),
        }
    }

    fn put(&self, key: &str, value: Value) -> Result<Value, FlError> {
        match self.name_to_index.get(key) {
            Some(_) => Err(FlError::new(format!("can't change parameter value {}", key))),
            None => self.map_scope.put(key, value),
        }
    }

    fn replace(&self, key: &str, new_value: Value) -> Result<Value, FlError> {
        match self.name_to_index.get(key) {
            Some(_) => Err(FlError::new(format!("can't change parameter value {}", key))),
            None => self.map_scope.replace(key, new_value),
        }
    }

    fn replace_local(&self, key: &str, new_value: Value) -> Result<Value, FlError> {
        match self.name_to_index.get(key) {
            Some(_) => Err(FlError::new(format!("can't change parameter value {}", key))),
            None => self.map_scope.replace(key, new_value),
        }
    }

    fn define(&self, key: &str, value: Value) -> Result<Value, FlError> {
        match self.name_to_index.get(key) {
            Some(_) => Err(FlError::new(format!("can't change parameter value {}", key))),
            None => self.map_scope.define(key, value),
        }
    }

    fn entries(&self) -> Vec<ScopeEntry> {
        // TODO: add parameter entries
        self.map_scope.entries()
    }
}
