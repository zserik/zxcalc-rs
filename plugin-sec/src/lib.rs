extern crate zxcalc_common;
use zxcalc_common::*;

#[derive(Debug, Default)]
pub struct SecPlugin;

impl Plugin for SecPlugin {
    fn calc(&self, x: f64) -> f64 {
        x / 60.0
    }
    fn calcinv(&self, x: f64) -> f64 {
        (x - x.trunc()) * 60.0
    }
}

declare_plugin!(SecPlugin, SecPlugin::default);
