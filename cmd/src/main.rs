use std::env;
use std::str::FromStr;

mod my;
mod lib;

fn function() {
  println!("called `function()`");
}

fn main() {
  my::function();

  function();

  my::indirect_access();

  my::nested::function();

  let mut numbers = Vec::new();
  for arg in env::args().skip(1) {
    numbers.push(u64::from_str(&arg).expect("error parsing argument"));
  }

  if numbers.len() == 0 {
    panic!("Error: No numbers provided")
  }

  let mut result = numbers[0];
  for i in 1..numbers.len() {
    result = lib::math::gcd(result, numbers[i]);
  }

  println!("The greatest common divisor of the numbers of {:?} is {}", numbers, result);
}
