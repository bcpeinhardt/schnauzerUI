use std::collections::HashMap;

pub struct Environment(HashMap<String, String>);

impl Environment {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn set_variable(&mut self, name: String, value: String) { 
        self.0.insert(name, value);
    }

    pub fn get_variable(&self, name: String) -> Option<String> { 
        self.0.get(&name).map(|s| s.clone() )
    }
}