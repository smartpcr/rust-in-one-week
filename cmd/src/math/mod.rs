use serde::Deserialize;

pub mod operations {
    pub fn gcd(mut n: u64, mut m: u64) -> u64 {
        assert!(n != 0 && m != 0);
        while m != 0 {
            if m < n {
                let temp = m;
                m = n;
                n = temp;
            }
            m %= n;
        }
        n
    }
}

#[derive(Deserialize)]
pub struct GcdParameters {
    pub n: u64,
    pub m: u64,
}
