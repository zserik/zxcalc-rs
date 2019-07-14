extern crate zxcalc_common;
use zxcalc_common::*;

#[derive(Debug, Default)]
pub struct Base10Plugin;

impl Plugin for Base10Plugin {
    fn calc(&self, x: f64) -> f64 {
        (10.0_f64).powf(x)
    }
    fn calcinv(&self, x: f64) -> f64 {
        x.log10()
    }
}

declare_plugin!(Base10Plugin, Base10Plugin::default);
