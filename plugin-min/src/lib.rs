extern crate zxcalc_common;
use zxcalc_common::*;

#[derive(Debug, Default)]
pub struct MinPlugin;

impl Plugin for MinPlugin {
    fn calc(&self, x: f64) -> f64 {
        x
    }
    fn calcinv(&self, x: f64) -> f64 {
        zx_modulo(x, 60.0)
    }
}

declare_plugin!(MinPlugin, MinPlugin::default);
