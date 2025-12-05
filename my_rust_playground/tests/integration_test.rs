use my_rust_playground::*;

#[test]
fn adding_positives() {
    let result = math::add(5, 6);
    assert_eq!(result, 11)
}

#[test]
fn mult_positives() {
    let result = math::mult(5, 6);
    assert_eq!(result, 30)
}
