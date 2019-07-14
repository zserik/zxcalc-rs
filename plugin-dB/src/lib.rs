#![allow(non_snake_case)]

extern crate zxcalc_common;
use zxcalc_common::*;

#[derive(Debug, Default)]
pub struct DBPlugin;

impl Plugin for DBPlugin {
    fn calc(&self, x: f64) -> f64 {
        x.log10() * 10.0
    }
    fn calcinv(&self, x: f64) -> f64 {
        (10.0_f64).powf(x / 10.0)
    }
}

declare_plugin!(DBPlugin, DBPlugin::default);
