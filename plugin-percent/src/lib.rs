extern crate zxcalc_common;
use zxcalc_common::*;

#[derive(Debug, Default)]
pub struct PercentPlugin;

impl Plugin for PercentPlugin {
    fn calc(&self, x: f64) -> f64 {
        1.0 + x / 100.0
    }
    fn calcinv(&self, x: f64) -> f64 {
        (x - 1.0) * 100.0
    }
}

declare_plugin!(PercentPlugin, PercentPlugin::default);
