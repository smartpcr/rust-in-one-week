pub(crate) fn gcd(mut n: u64, mut m: u64) -> u64 {
  assert!(n != 0 && m != 0);
  while m != n {
    if m < n {
      let t = m;
      m = n;
      n = t;
    }
    m = m % n;
  }
  m // Return the final value of m
}

#[cfg(test)]
mod test {
  #[test]
  fn test_gcd() {
    assert_eq!(super::gcd(14, 15), 1);
    assert_eq!(super::gcd(2 * 3 * 5 * 11 * 17, 3 * 7 * 11 * 13 * 19), 3 * 11);
  }
}