pub fn zx_modulo(mut x: f64, y: f64) -> f64 {
    x /= y;
    (x - x.trunc()) * y
}

pub fn zx_sgnprop(v: f64, s: f64) -> f64 {
    v.copysign(v * s)
}

pub trait Plugin: std::any::Any + Send + Sync {
    fn calc(&self, x: f64) -> f64;
    fn calcinv(&self, x: f64) -> f64;
}

#[macro_export]
macro_rules! declare_plugin {
    ($plgt:ty, $constructor:path) => {
        #[no_mangle]
        pub extern "C" fn _plugin_create() -> *mut $crate::Plugin {
            // make sure the constructor is the correct type.
            let constructor: fn() -> $plgt = $constructor;
            let object = constructor();
            let boxed: Box::<$crate::Plugin> = Box::new(object);
            Box::into_raw(boxed)
        }
    };
}
