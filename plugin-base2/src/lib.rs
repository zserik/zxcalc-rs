extern crate zxcalc_common;
use zxcalc_common::*;

#[derive(Debug, Default)]
pub struct Base2Plugin;

impl Plugin for Base2Plugin {
    fn calc(&self, x: f64) -> f64 {
        x.exp2()
    }
    fn calcinv(&self, x: f64) -> f64 {
        x.log2()
    }
}

declare_plugin!(Base2Plugin, Base2Plugin::default);
