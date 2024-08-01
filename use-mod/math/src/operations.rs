pub fn add(left: f64, right: f64) -> f64 {
    left + right
}

pub fn subtract(left: f64, right: f64) -> f64 {
    left - right
}

pub fn multiply(left: f64, right: f64) -> f64 {
    left * right
}

pub fn divide(left: f64, right: f64) -> f64 {
    left / right
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

#[cfg(test)]
mod test {
    use crate::numbers::get_two_numbers;
    use super::*;
    
    const A: f64 = 2.0;
    const B: f64 = 2.0;

    #[test]
    fn test_add_should_work() {
        let result = add(A, B);
        assert_eq!(result, 4f64);
    }

    #[test]
    fn test_subtract_should_work() {
        let result = subtract(A, B);
        assert_eq!(result, 0f64);
    }

    #[test]
    fn test_multiply_should_work() {
        let result = multiply(A, B);
        assert_eq!(result, 4f64);
    }

    #[test]
    fn test_divide_should_work() {
        let result = divide(A, B);
        assert_eq!(result, 1f64);
    }

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(14, 15), 1);
        assert_eq!(gcd(2 * 3 * 5 * 11 * 17, 3 * 7 * 11 * 13 * 19), 3 * 11);
    }
}
