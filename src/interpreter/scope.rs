use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::{FlError, Value};


pub trait Scope: Send + Sync {

    /// Get next Scope in chain (if there is a next).
    fn get_next(&self) -> Option<Arc<dyn Scope>>;

    /// Name of this Scope
    fn get_name(&self) -> String;

    /// Lookup a symbol, if not found in this scope, walk chain of scopes.
    /// Returns Value::Nil if key is not found in the whole chain.
    fn get(&self, key: &str) -> Value;

    /// Put a key value mapping in the scope, may overwrite an existing mapping.
    /// Returns the old mapping for this key (or Nil).
    /// Removes the mapping if value is Value::Nil. 
    fn put(&self, key: &str, value: Value) -> Value;

    /// Replace a value in this scope, if not there, search through scope chain until
    /// value is found. Returns an Error if there is no mapping in the whole chain.
    /// Returns the old mapping for this key.
    fn replace(&self, key: &str, new_value: Value) -> Result<Value, FlError>;

    /// Replace a value in this scope without walking up the chain.
    /// Returns the old mapping for this key.
    fn replace_local(&self, key: &str, new_value: Value) -> Result<Value, FlError>;

    /// Set a value in this scope. 
    /// Returns an error when this scope already has a mapping for the key.
    /// Returns a copy of the value
    fn define(&self, key: &str, value: Value) -> Result<Value, FlError>;
/*
	/**
	 * Put a key value mapping in the scope, may overwrite an existing mapping. The
	 * name is taken from the value.
	 * 
	 * @param value The value of the symbol, null values are allowed and will remove
	 *              the mapping.
	 */
	public void define(Named value) throws ScopeException;

	@Override
	public Iterator<Entry<String, FplValue>> iterator();
	
	public Set<String> keySet();
	public Collection<FplValue> values();
	public Set<Entry<String, FplValue>> entrieSet();
 */
}

/// A Scope backed by a HashMap. The map is protected by a RwLock (interior mutability),
/// so all Scope methods work with a shared reference and the scope can be shared via Arc.
/// Value::Nil is never stored in the map.
pub struct MapScope {
    name: String,
    next: Option<Arc<dyn Scope>>,
    map: RwLock<HashMap<String, Value>>,
}

impl MapScope {
    pub fn new(name: &str, next: Option<Arc<dyn Scope>>) -> MapScope {
        MapScope {
            name: String::from(name),
            next,
            map: RwLock::new(HashMap::new()),
        }
    }

    /// Replace the mapping for key in this scope only. Returns None when there is no
    /// mapping for the key. A Nil value removes the mapping.
    fn replace_in_map(&self, key: &str, new_value: Value) -> Option<Value> {
        let mut map = self.map.write().expect("scope lock poisoned");
        if !map.contains_key(key) {
            return None;
        }
        if let Value::Nil = new_value {
            map.remove(key)
        } else {
            map.insert(String::from(key), new_value)
        }
    }

    fn no_mapping(key: &str) -> FlError {
        FlError::new(&format!("No mapping for \"{}\"", key))
    }
}

impl Scope for MapScope {

    fn get_next(&self) -> Option<Arc<dyn Scope>> {
        self.next.clone()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get(&self, key: &str) -> Value {
        // Release the lock before walking up the chain.
        let local = self.map.read().expect("scope lock poisoned").get(key).cloned();
        match (local, &self.next) {
            (Some(value), _) => value,
            (None, Some(next)) => next.get(key),
            (None, None) => Value::Nil,
        }
    }

    fn put(&self, key: &str, value: Value) -> Value {
        let mut map = self.map.write().expect("scope lock poisoned");
        let old = if let Value::Nil = value {
            map.remove(key)
        } else {
            map.insert(String::from(key), value)
        };
        old.unwrap_or(Value::Nil)
    }

    fn replace(&self, key: &str, new_value: Value) -> Result<Value, FlError> {
        // Try locally first. The write lock is released before the next scope is asked.
        match self.replace_in_map(key, new_value.clone()) {
            Some(old) => Ok(old),
            None => match &self.next {
                Some(next) => next.replace(key, new_value),
                None => Err(Self::no_mapping(key)),
            },
        }
    }

    fn replace_local(&self, key: &str, new_value: Value) -> Result<Value, FlError> {
        self.replace_in_map(key, new_value).ok_or_else(|| Self::no_mapping(key))
    }

    fn define(&self, key: &str, value: Value) -> Result<Value, FlError> {
        let mut map = self.map.write().expect("scope lock poisoned");
        if map.contains_key(key) {
            return Err(FlError::new(&format!("\"{}\" is already defined in scope \"{}\"", key, self.name)));
        }
        if !matches!(value, Value::Nil) {
            map.insert(String::from(key), value.clone());
        }
        Ok(value)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn int(i: i64) -> Value {
        Value::Integer(i)
    }

    #[test]
    fn create() {
        let scope = MapScope::new("global", None);
        assert_eq!("global", scope.get_name());
        assert!(scope.get_next().is_none());
        assert_eq!(Value::Nil, scope.get("a"));
    }

    #[test]
    fn put_and_get() {
        let scope = MapScope::new("global", None);
        assert_eq!(Value::Nil, scope.put("a", int(1)));
        assert_eq!(int(1), scope.get("a"));
        assert_eq!(int(1), scope.put("a", int(2)));
        assert_eq!(int(2), scope.get("a"));
    }

    #[test]
    fn put_nil_removes() {
        let scope = MapScope::new("global", None);
        scope.put("a", int(1));
        assert_eq!(int(1), scope.put("a", Value::Nil));
        assert_eq!(Value::Nil, scope.get("a"));
        assert_eq!(Value::Nil, scope.put("a", Value::Nil));
    }

    #[test]
    fn get_walks_chain() {
        let global = Arc::new(MapScope::new("global", None));
        global.put("a", int(1));
        global.put("b", int(2));
        let local = MapScope::new("local", Some(global.clone()));
        local.put("b", int(3));

        assert_eq!(int(1), local.get("a"));
        assert_eq!(int(3), local.get("b"));
        assert_eq!(int(2), global.get("b"));
        assert_eq!(Value::Nil, local.get("c"));
        assert_eq!("global", local.get_next().unwrap().get_name());
    }

    #[test]
    fn replace_walks_chain() {
        let global = Arc::new(MapScope::new("global", None));
        global.put("a", int(1));
        let local = MapScope::new("local", Some(global.clone()));

        assert_eq!(Ok(int(1)), local.replace("a", int(2)));
        assert_eq!(int(2), global.get("a"));
        assert!(local.map.read().unwrap().is_empty());
    }

    #[test]
    fn replace_prefers_local() {
        let global = Arc::new(MapScope::new("global", None));
        global.put("a", int(1));
        let local = MapScope::new("local", Some(global.clone()));
        local.put("a", int(2));

        assert_eq!(Ok(int(2)), local.replace("a", int(3)));
        assert_eq!(int(3), local.get("a"));
        assert_eq!(int(1), global.get("a"));
    }

    #[test]
    fn replace_missing_is_error() {
        let global = Arc::new(MapScope::new("global", None));
        let local = MapScope::new("local", Some(global));
        assert!(local.replace("a", int(1)).is_err());
        assert_eq!(Value::Nil, local.get("a"));
    }

    #[test]
    fn replace_nil_removes() {
        let scope = MapScope::new("global", None);
        scope.put("a", int(1));
        assert_eq!(Ok(int(1)), scope.replace("a", Value::Nil));
        assert_eq!(Value::Nil, scope.get("a"));
    }

    #[test]
    fn replace_local_does_not_walk_chain() {
        let global = Arc::new(MapScope::new("global", None));
        global.put("a", int(1));
        let local = MapScope::new("local", Some(global.clone()));

        assert!(local.replace_local("a", int(2)).is_err());
        assert_eq!(int(1), global.get("a"));

        local.put("a", int(3));
        assert_eq!(Ok(int(3)), local.replace_local("a", int(4)));
        assert_eq!(int(4), local.get("a"));
    }

    #[test]
    fn define() {
        let global = Arc::new(MapScope::new("global", None));
        global.put("a", int(1));
        let local = MapScope::new("local", Some(global.clone()));

        // Shadowing a mapping of an outer scope is allowed.
        assert_eq!(Ok(int(2)), local.define("a", int(2)));
        assert_eq!(int(2), local.get("a"));
        assert_eq!(int(1), global.get("a"));

        // Defining twice in the same scope is not.
        assert!(local.define("a", int(3)).is_err());
        assert_eq!(int(2), local.get("a"));
    }

    #[test]
    fn shared_between_threads() {
        let scope = Arc::new(MapScope::new("global", None));
        let handles: Vec<_> = (0..8)
            .map(|i| {
                let scope = scope.clone();
                std::thread::spawn(move || {
                    scope.put(&format!("key{}", i), int(i));
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }
        for i in 0..8 {
            assert_eq!(int(i), scope.get(&format!("key{}", i)));
        }
    }
}