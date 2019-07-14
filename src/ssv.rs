use sortedvec::sortedvec;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn ssv_strhash(x: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}

sortedvec! {
    pub struct SSV {
        fn key_deriv(x: &String) -> u64 {
            ssv_strhash(x)
        }
    }
}

pub fn ssv_contains(v: &SSV, x: &str) -> bool {
    v.contains(&ssv_strhash(x))
}

macro_rules! ssv_create {
    ($($elem:expr),+) => {
        SSV::from(vec![
            $($elem.to_string(),)+
        ])
    }
}
