
#[test]
fn tracked_type() {
    let val: i8 = 42;
    let b = val != 0;
    assert!(b);
}
