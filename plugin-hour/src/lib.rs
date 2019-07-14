extern crate zxcalc_common;
use zxcalc_common::*;

#[derive(Debug, Default)]
pub struct HourPlugin;

impl Plugin for HourPlugin {
    fn calc(&self, x: f64) -> f64 {
        x * 60.0
    }
    fn calcinv(&self, x: f64) -> f64 {
        zx_modulo(x / 60.0, 24.0)
    }
}

declare_plugin!(HourPlugin, HourPlugin::default);
