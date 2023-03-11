use std::env;
use std::str::FromStr;

use lib::math::gcd;

fn main() {
  let mut numbers = Vec::new();
  for arg in env: args().skip(1) {
    numbers.push(arg.parse::<u64>().expect("error parsing argument"));
  }

  if numbers.len() == 0 {
    eprintln!("Usage: gcd NUMBER...");
    std::process::exit(1);
  }

  let mut d = numbers[0];
  for m in &numbers[1..] {
    d = gcd(d, *m);
  }

  println!("The greatest common divisor of {:?} is {}", numbers, d);
}
