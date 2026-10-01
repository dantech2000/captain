use super::{bytes_label, percent_label, rate_label, short_bytes_label, short_percent_label};

#[test]
fn bytes_pick_the_largest_unit() {
    assert_eq!(bytes_label(512), "512 B");
    assert_eq!(bytes_label(1536), "1.5 KB");
    assert_eq!(bytes_label(182 * 1024 * 1024), "182 MB");
    assert_eq!(bytes_label(1288 * 1024 * 1024), "1.3 GB");
}

#[test]
fn rates_append_per_second() {
    assert_eq!(rate_label(8294), "8.1 KB/s");
}

#[test]
fn percents_drop_decimals_above_ten() {
    assert_eq!(percent_label(3.06), "3.1%");
    assert_eq!(percent_label(42.4), "42%");
}

#[test]
fn negative_and_negative_zero_show_as_zero() {
    assert_eq!(percent_label(-0.0), "0.0%");
    assert_eq!(percent_label(-0.3), "0.0%");
}

#[test]
fn short_labels_keep_a_decimal_only_below_ten() {
    assert_eq!(short_bytes_label(55_400_000), "53 MB");
    assert_eq!(short_bytes_label(1_717_986_918), "1.6 GB");
    assert_eq!(short_percent_label(0.4), "0%");
    assert_eq!(short_percent_label(-0.3), "0%");
}
