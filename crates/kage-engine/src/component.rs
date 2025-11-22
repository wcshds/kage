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
    pub fn set<S1, S2>(&mut self, name: S1, data: S2) -> bool
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.hash.insert(name.into(), data.into()).is_some()
    }

    /// Adds or updates an element with the given glyph name and KAGE data.
    /// It is an alias for the `set` method.
    pub fn push<S1, S2>(&mut self, name: S1, data: S2) -> bool
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.hash.insert(name.into(), data.into()).is_some()
    }

    /// Searches the store for the given glyph name and returns the corresponding
    /// KAGE data.
    pub fn search(&self, name: &str) -> Option<&str> {
        self.hash.get(name).map(|x| x.as_str())
    }

    pub fn len(&self) -> usize {
        self.hash.len()
    }
}
