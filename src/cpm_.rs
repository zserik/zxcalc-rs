use libloading::Library;
use hashbrown::HashMap;

struct PluginHandle {
    lib: Option<Library>,
    scale: isize,
}

type CowString<'a> = std::borrow::Cow<'a, str>;

pub struct CalcPluginManager {
    plugins: HashMap<String, PluginHandle>,
    aliases: HashMap<String, String>,
}

const CPM_LDPATHS: &[&str] = &["../plugins/target/release", "./zxcalc.plugins"];

impl CalcPluginManager {
    pub fn new() -> Self {
        Self { plugins: HashMap::new(), aliases: HashMap::new() }
    }

    pub fn alias(&mut self, aln: &str, plgn: &str) {
        self.aliases.insert(aln.to_string(), plgn.to_string());
    }

    pub fn resolve<'a>(&'a mut self, plgn: &'a str) -> Option<CowString<'a>> {
        let mut ret: CowString = plgn.into();

        // 1. check for aliases
        if let Some(r_a) = self.aliases.get(plgn) {
            ret = r_a.into();
        }

        // 2. check if already loaded
        if let Some(y) = self.plugins.get(&*ret) {
            if y.lib.is_some() {
                return Some(ret);
            }
        }

        // 3. check for plugin as shared-object
        for i in CPM_LDPATHS {
            let pp = i.to_string() + "/lib" + &*ret + ".so";
            let lh = match Library::new(pp).ok() {
                Some(lh) => lh,
                None => continue,
            };
            self.plugins.insert(ret.to_string(), PluginHandle { lib: Some(lh), scale: 1 });
        }
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

    pub fn set_scale(&mut self, plgn: &str, mut scval: isize) {
        if scval == 0 {
            scval = 1;
        }
        if let Some(x) = self.plugins.get_mut(plgn) {
            x.scale = scval;
        }
    }

    pub fn calc(&self, plgn: &str, xval: f64) -> Option<f64> {
        None
    }
}

impl Drop for PluginHandle {
    fn drop(&mut self) {
        if let Some(x) = self.lib.take() {
            drop(x);
        }
    }
}

impl Drop for CalcPluginManager {
    fn drop(&mut self) {
        for (k, v) in self.plugins.drain() {
            drop(v);
            drop(k);
        }
    }
}
