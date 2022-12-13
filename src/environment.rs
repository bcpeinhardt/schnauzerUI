use std::collections::HashMap;

/// Represents the "state" of the programs execution. Basically
/// keeps charge of variables and their values.
pub struct Environment(HashMap<String, String>);

impl Environment {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Set a variable value. SchnauzerUI makes no distinction between
    /// declaration and instantiation.
    pub fn set_variable(&mut self, name: String, value: String) {
        self.0.insert(name, value);
    }

    /// Get the value of a variable if it exists, or None
    /// if it does not.
    pub fn get_variable(&self, name: &str) -> Option<String> {
        self.0.get(name).map(|s| s.clone())
    }
}
