use libloading::Library;
use hashbrown::HashMap;

struct PluginHandle {
    lib: Library,
    scale: usize,
}

pub struct CalcPluginManager {
    plugins: HashMap<String, PluginHandle>,
    aliases: HashMap<String, String>,
}

impl CalcPluginManager {
    pub fn new() -> Self {
        Self { plugins: HashMap::new(), aliases: HashMap::new() }
    }

    pub fn resolve<'a>(&self, plgn: &'a str) -> Option<std::borrow::Cow<'a, str>> {
        None
    }

    pub fn list_loaded_plugins(&self) {
        for i in self.plugins.keys() {
            println!("\t  {}", i);
        }
        for (k, v) in self.aliases.iter() {
            println!("\tA {}\t= {}", k, v);
        }
    }

    pub fn set_scale(&self, plgn: &str, scval: f64) {}

    pub fn calc(&self, plgn: &str, xval: f64) -> Option<f64> {
        None
    }
}
