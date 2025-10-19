use std::collections::HashMap;

struct Component {
    hash: HashMap<String, String>,
}

impl Component {
    fn new() -> Self {
        Component {
            hash: HashMap::new(),
        }
    }

    /// Adds or updates an element with the given glyph name and KAGE data.
    fn set(&mut self, name: String, data: String) -> bool {
        self.hash.insert(name, data).is_some()
    }

    /// Adds or updates an element with the given glyph name and KAGE data.
    /// It is an alias for the `set` method.
    fn push(&mut self, name: String, data: String) -> bool {
        self.hash.insert(name, data).is_some()
    }

    /// Searches the store for the given glyph name and returns the corresponding
    /// KAGE data.
    fn search(&self, name: &str) -> &str {
        &self.hash[name]
    }
}
