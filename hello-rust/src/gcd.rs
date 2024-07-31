pub(crate) fn gcd(n: u64, m: u64) -> u64 {
  if m == 0 {
    return n;
  }
  gcd(m, n % m)
}

#[cfg(test)]
mod test {
  #[test]
  fn test_gcd() {
    assert_eq!(super::gcd(14, 15), 1);
    assert_eq!(super::gcd(2 * 3 * 5 * 11 * 17, 3 * 7 * 11 * 13 * 19), 3 * 11);
  }
}