pub mod math {

    pub fn add(left: u8, right: u8) -> u16 {
        (left as u16) + (right as u16)
    }

    pub fn mult(left: u8, right: u8) -> u16 {
        (left as u16) * (right as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::math::*;

    #[test]
    fn add_positive() {
        let result = add(2, 3);
        assert_eq!(result, 5);
    }

    #[test]
    fn add_max_values() {
        let result = add(255, 255);
        assert_eq!(result, 510);
    }

    #[test]
    fn mult_positives() {
        let result = mult(5, 20);
        assert_eq!(result, 100);
    }

    #[test]
    fn mult_max_positives() {
        let result = mult(255, 255);
        assert_eq!(result, 255 * 255);
    }
}
