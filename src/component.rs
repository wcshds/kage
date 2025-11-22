use std::collections::HashMap;

pub struct Components {
    hash: HashMap<String, String>,
}

impl Components {
    pub(crate) fn new() -> Self {
        Components {
            hash: HashMap::new(),
        }
    }

    /// Adds or updates an element with the given glyph name and KAGE data.
    pub fn set<S>(&mut self, name: S, data: S) -> bool
    where
        S: Into<String>,
    {
        self.hash.insert(name.into(), data.into()).is_some()
    }

    /// Adds or updates an element with the given glyph name and KAGE data.
    /// It is an alias for the `set` method.
    pub fn push<S>(&mut self, name: S, data: S) -> bool
    where
        S: Into<String>,
    {
        self.hash.insert(name.into(), data.into()).is_some()
    }

    /// Searches the store for the given glyph name and returns the corresponding
    /// KAGE data.
    pub fn search(&self, name: &str) -> Option<&str> {
        self.hash.get(name).map(|x| x.as_str())
    }
}
