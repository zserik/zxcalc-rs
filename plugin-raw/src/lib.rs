extern crate zxcalc_common;
use zxcalc_common::*;

#[derive(Debug, Default)]
pub struct RawPlugin;

impl Plugin for RawPlugin {
    fn calc(&self, x: f64) -> f64 {
        x
    }
    fn calcinv(&self, x: f64) -> f64 {
        x
    }
}

declare_plugin!(RawPlugin, RawPlugin::default);
