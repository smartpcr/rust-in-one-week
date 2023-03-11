use serde::Deserialize;

#[derive(Deserialize)]
pub struct GcdParameters {
  n: u64,
  m: u64,
}

pub fn gcd(a: u64, b: u64) -> u64 {
    let mut a = a;
    let mut b = b;
    while b != 0 {
        let temp = a % b;
        a = b;
        b = temp;
    }
    a
}

mod tests {
    use super::*;

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(14, 15), 1);
        assert_eq!(gcd(2 * 3 * 5 * 11 * 17, 3 * 7 * 11 * 13 * 19), 3 * 11);
    }
}
