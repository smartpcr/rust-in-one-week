use serde::Deserialize;

#[derive(Deserialize)]
pub struct GcdParameters {
    pub n: u64,
    pub m: u64,
}

pub fn get_two_numbers() -> (u64, u64) {
    (2, 2)
}

pub fn get_two_random_numbers() -> (f64, f64) {
    let numbers: Vec<f64> = fastrand::choose_multiple(1..100, 2)
        .iter()
        .map(|n| *n as f64)
        .collect();
    (numbers[0], numbers[1])
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_two_numbers_should_return_2_and_2() {
        let (left, right) = get_two_numbers();
        assert_eq!(left, 2);
        assert_eq!(right, 2);
    }
}
