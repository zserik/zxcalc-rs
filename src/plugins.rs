pub fn zx_modulo(mut x: f64, y: f64) -> f64 {
    if x < y && 0.0 <= x {
        x
    } else {
        x /= y;
        (x - x.trunc()) * y
    }
}

/*
pub fn zx_sgnprop(v: f64, s: f64) -> f64 {
    v.copysign(v * s)
}

pub fn invperc_wrap(x: f64, cond: bool) -> f64 {
    let x2 = percent::calc(x);
    if cond { 1.0 / x2 } else { x2 }
}
*/

pub mod scaling {
    pub fn calc(x: i64) -> f64 {
        if x >= 0 {
            x as f64
        } else {
            -1.0 / (x as f64)
        }
    }

    /*
    pub fn calcinv(x: f64) -> i64 {
        if x == 0.0 {
            0
        } else if (x < 1.0) && (0.0 < x) {
            (-1.0 / x).round() as i64
        } else {
            x.round() as i64
        }
    }
    */
}

pub mod raw {
    #[doc(no_inline)]
    pub use core::convert::identity as calc;
    #[doc(no_inline)]
    pub use core::convert::identity as calcinv;
}

#[derive(Clone, Copy)]
pub struct Plugin {
    pub calc: fn(f64) -> f64,
    pub calcinv: fn(f64) -> f64,
}

macro_rules! bareplug {
    ($x:ident, $($name:ident, $calc:expr, $calcinv:expr),* $(,)?) => {
        $(
        #[allow(non_snake_case)]
        pub mod $name {
            #[allow(unused_imports)]
            use super::*;

            pub fn calc($x: f64) -> f64 {
                $calc
            }
            pub fn calcinv($x: f64) -> f64 {
                $calcinv
            }
        }
        )*
    }
}

macro_rules! plugins {
    ($($name:expr => $mod:tt),* $(,)?) => {
        pub static PLUGINS: phf::Map<&'static str, Plugin> = phf::phf_map! {
            $($name => Plugin { calc: ($mod :: calc), calcinv: ($mod :: calcinv) }),*
        };
    }
}

bareplug! { x,
    base10,	(10.0_f64).powf(x),	x.log10(),
    base2,	x.exp2(),		x.log2(),
    dB,		x.log10() * 10.0,	(10.0_f64).powf(x / 10.0),
    percent,	1.0 + x / 100.0,	(x - 1.0) * 100.0,

    day,	x * 1440.0,		x / 1440.0,
    hour,	x * 60.0,		zx_modulo(x / 60.0, 24.0),
    min,	x,			zx_modulo(x, 60.0),
    sec,	x / 60.0,		(x - x.trunc()) * 60.0,
}

plugins! {
    "_" => raw,
    "%" => percent,
    "2^" => base2,
    "10^" => base10,
    "dB" => dB,
    "day" => day,
    "hour" => hour,
    "min" => min,
    "sec" => sec,
}
