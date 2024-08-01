pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub fn subtract(left: u64, right: u64) -> u64 {
    left - right
}

pub fn multiply(left: u64, right: u64) -> u64 {
    left * right
}

pub fn divide(left: u64, right: u64) -> u64 {
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
mod tests {
    use super::*;

    #[test]
    fn add_should_work() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn subtract_should_work() {
        let result = subtract(2, 2);
        assert_eq!(result, 0);
    }

    #[test]
    fn multiply_should_work() {
        let result = multiply(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn divide_should_work() {
        let result = divide(2, 2);
        assert_eq!(result, 1);
    }

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(14, 15), 1);
        assert_eq!(gcd(2 * 3 * 5 * 11 * 17, 3 * 7 * 11 * 13 * 19), 3 * 11);
    }
}
