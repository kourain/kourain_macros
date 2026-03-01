use kourain_macro::PropertyTracked;

#[derive(Debug, PropertyTracked)]
struct NguoiDung {
    ten: String,
    tuoi: u32,
    is_changed: bool,
}

#[test]
fn tracked_setters_and_reset() {
    let mut nguoi1 = NguoiDung {
        ten: "An".to_string(),
        tuoi: 25,
        is_changed: false,
    };
    // Macro should generate these setters
    nguoi1.set_tuoi(26);
    nguoi1.set_ten("Bình".to_string());

    assert!(nguoi1.is_changed, "expected is_changed to be true after setters");
    assert_eq!(nguoi1.ten, "Bình");
    assert_eq!(nguoi1.tuoi, 26);

    nguoi1.reset_changed();
    assert!(!nguoi1.is_changed, "expected is_changed to be false after reset");
}
