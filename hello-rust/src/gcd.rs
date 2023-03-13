fn gcd(mut n: u64, mut m: u64) -> u64 {
  assert!(n != 0 && m != 0);
  while m != n {
    if m < n {
      let t = m;
      m = n;
      n = t;
    }
    m = m % n;
  }
}

#[cfg(test)]
mod test {
  #[test]
  fn test_gcd() {}
}