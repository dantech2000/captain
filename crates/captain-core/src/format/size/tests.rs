use super::{bytes_label, percent_label, rate_label};

#[test]
fn bytes_and_rates_pick_the_largest_unit() {
    assert_eq!(bytes_label(512), "512 B");
    assert_eq!(bytes_label(1536), "1.5 KB");
    assert_eq!(bytes_label(182 * 1024 * 1024), "182 MB");
    assert_eq!(bytes_label(1288 * 1024 * 1024), "1.3 GB");
    assert_eq!(rate_label(8294), "8.1 KB/s");
}

#[test]
fn percents_drop_decimals_above_ten_and_never_go_negative() {
    assert_eq!(percent_label(3.06), "3.1%");
    assert_eq!(percent_label(42.4), "42%");
    assert_eq!(percent_label(-0.0), "0.0%");
    assert_eq!(percent_label(-0.3), "0.0%");
}
