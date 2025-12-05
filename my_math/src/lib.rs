pub fn add(left: u8, right: u8) -> u16 {
    (left as u16) + (right as u16)
}

pub fn mult(left: u8, right: u8) -> u16 {
    (left as u16) * (right as u16)
}

fn private_div(left: u8, right: u8) -> u8 {
    left / right
}

#[cfg(test)]
mod tests {

    use super::*;

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

    #[test]
    fn div_exact() {
        let result = private_div(25, 5);
        assert_eq!(result, 5);
    }

    #[test]
    fn div_rounded() {
        let result = private_div(25, 4);
        assert_eq!(result, 6);
    }
}
