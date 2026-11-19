use ocf_core::FidelityClass;

#[test]
fn ordering_and_satisfies() {
    assert!(FidelityClass::F0 < FidelityClass::F3);
    assert!(FidelityClass::satisfies(FidelityClass::F0, FidelityClass::F0));
    assert!(FidelityClass::satisfies(FidelityClass::F0, FidelityClass::F3));
    assert!(!FidelityClass::satisfies(FidelityClass::F3, FidelityClass::F0));
    assert!(FidelityClass::F0.is_lossless());
    assert!(!FidelityClass::F3.is_lossless());
    assert_eq!(FidelityClass::from_u8(2), Some(FidelityClass::F2));
    assert_eq!(FidelityClass::from_u8(9), None);
}
