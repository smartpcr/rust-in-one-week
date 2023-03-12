use std::str::FromStr;
use num::Complex;

pub mod mandelbrot {
    fn square_loop(mut x: f64) {
        loop {
            x = x * x;
        }
    }

    fn square_add_loop(c: f64) {
        let mut x = 0.0;
        loop {
            x = x * x + c;
        }
    }

    fn complex_square_add_loop(c: Complex<f64>) {
        let mut z = Complex::new(0.0, 0.0);
        loop {
            z = z * z + c;
        }
    }

    /// Determine if `c` is in the Mandelbrot set, using at most `limit` iterations to decide.
    ///
    /// if `c` is not a member, return `Some(i)`, where `i` is the number of
    /// iterations it took for `c` to leave the circle of radius 2 centered on the origin.
    /// if `c` seems to be a member (more precisely, if we reached the iteration limit
    /// without being able to prove that `c` is not a member), return `None`.
    fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
        let mut z = Complex::new(0.0, 0.0);
        for i in 0..limit {
            z = z * z + c;
            if z.norm_sqr() > 4.0 {
                return Some(i);
            }
        }
        None
    }

    pub fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
        match s.find(separator) {
            None => None,
            Some(index) => {
                match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
                    (Ok(l), Ok(r)) => Some((l, r)),
                    _ => None,
                }
            }
        }
    }
}

pub use mandelbrot::*;
