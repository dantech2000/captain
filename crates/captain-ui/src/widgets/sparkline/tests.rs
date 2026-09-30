use super::ease;

#[test]
fn a_one_sample_burst_eases_into_a_low_rise_and_fall() {
    let eased = ease(&[0.0, 0.0, 2.5, 0.0, 0.0]);
    assert_eq!(eased[..2], [0.0, 0.0]);
    assert!(eased[2] > 0.0 && eased[2] < 2.5);
    assert!(eased[3] < eased[2] && eased[4] < eased[3] && eased[4] > 0.0);
}
