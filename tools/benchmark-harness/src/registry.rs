use crate::Error;
use crate::adapter::FrameworkAdapter;
use std::collections::HashMap;
use std::sync::Arc;

pub struct AdapterRegistry {
    adapters: HashMap<String, Arc<dyn FrameworkAdapter>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    pub fn register(&mut self, adapter: Arc<dyn FrameworkAdapter>) -> crate::Result<()> {
        let name = adapter.name().to_string();
        if self.adapters.contains_key(&name) {
            return Err(Error::Config(format!("Adapter '{}' already registered", name)));
        }
        self.adapters.insert(name, adapter);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn FrameworkAdapter>> {
        self.adapters.get(name).cloned()
    }

    pub fn adapters(&self) -> Vec<Arc<dyn FrameworkAdapter>> {
        self.adapters.values().cloned().collect()
    }

    pub fn names(&self) -> Vec<String> {
        self.adapters.keys().cloned().collect()
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}
