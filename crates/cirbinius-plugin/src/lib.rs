use std::collections::HashMap;

/// A registered optimization or lowering pass provided by a plugin.
pub trait PluginPass: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn version(&self) -> &'static str;
}

/// A plugin that can extend the compiler with custom passes.
pub trait CirbiniusPlugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn passes(&self) -> Vec<Box<dyn PluginPass>>;
}

/// Registry of loaded plugins.
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn CirbiniusPlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { plugins: HashMap::new() }
    }

    pub fn register(&mut self, plugin: Box<dyn CirbiniusPlugin>) {
        let name = plugin.name().to_string();
        self.plugins.insert(name, plugin);
    }

    pub fn list(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    pub fn get(&self, name: &str) -> Option<&dyn CirbiniusPlugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}
