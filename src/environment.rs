//! The "Environment" is where the interpreter keeps track of variable values.
//! As you can see, it's nothing fancy.

use std::collections::HashMap;

/// Represents the "state" of the programs execution. Basically
/// keeps track of variables and their values.
#[derive(Debug)]
pub struct Environment(HashMap<String, String>);

impl Environment {
    /// Creates a new environment
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Set a variable value. SchnauzerUI makes no distinction between
    /// declaration and instantiation.
    pub fn set_variable(&mut self, name: String, value: String) {
        let _ = self.0.insert(name, value);
    }

    /// Get the value of a variable if it exists, or None
    /// if it does not.
    pub fn get_variable(&self, name: &str) -> Option<String> {
        self.0.get(name).cloned()
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
