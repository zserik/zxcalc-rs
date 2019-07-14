pub fn zx_modulo(mut x: f64, y: f64) -> f64 {
    x /= y;
    (x - x.trunc()) * y
}

pub fn zx_sgnprop(v: f64, s: f64) -> f64 {
    v.copysign(v * s)
}
