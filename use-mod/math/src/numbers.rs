use serde::Deserialize;

#[derive(Deserialize)]
pub struct GcdParameters {
    pub n: u64,
    pub m: u64,
}

pub fn get_two_numbers() -> (u64, u64) {
    (2, 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_two_numbers_should_return_2_and_2() {
        let (left, right) = get_two_numbers();
        assert_eq!(left, 2);
        assert_eq!(right, 2);
    }
}