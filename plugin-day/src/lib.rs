extern crate zxcalc_common;
use zxcalc_common::*;

#[derive(Debug, Default)]
pub struct DayPlugin;

impl Plugin for DayPlugin {
    fn calc(&self, x: f64) -> f64 {
        x * 1440.0
    }
    fn calcinv(&self, x: f64) -> f64 {
        x / 1440.0
    }
}

declare_plugin!(DayPlugin, DayPlugin::default);
