use my_math::*;

#[test]
fn adding_positives() {
    let result = add(5, 6);
    assert_eq!(result, 11)
}
