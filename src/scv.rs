use sortedvec::sortedvec;

sortedvec! {
    pub struct SCV {
        fn derive_key(x: &char) -> char {
            *x
        }
    }
}

pub fn scv_contains(v: &SCV, x: char) -> bool {
    v.contains(&x)
}

pub fn scv_create(input: &str) -> SCV {
    SCV::from(input.chars().collect::<Vec<char>>())
}
