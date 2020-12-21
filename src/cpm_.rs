use std::collections::HashMap;
use zxcalc_plugins::Plugin;

pub struct CalcPluginManager {
    plgs: HashMap<&'static str, (Plugin, i64)>,
}

impl CalcPluginManager {
    pub fn new() -> Self {
        Self {
            plgs: zxcalc_plugins::PLUGINS.entries().map(|(&k, &v)| (k, (v, 1))).collect(),
        }
    }

    pub fn list_loaded_plugins(&self) {
        for i in self.plgs.keys() {
            println!("\t  {}", i);
        }
    }

    pub fn set_scale(&mut self, plgn: &str, mut scval: i64) {
        if scval == 0 {
            scval = 1;
        }
        if let Some(x) = self.plgs.get_mut(plgn) {
            x.1 = scval;
        }
    }

    pub fn calc(&self, plgn: &str, xval: f64) -> Option<f64> {
        let plg = self.plgs.get(plgn)?;
        Some((plg.0.calc)(xval) * zxcalc_plugins::scaling::calc(plg.1))
    }

    pub fn calcinv(&self, plgn: &str, xval: f64) -> Option<f64> {
        let plg = self.plgs.get(plgn)?;
        Some((plg.0.calcinv)(xval / zxcalc_plugins::scaling::calc(plg.1)))
    }
}
