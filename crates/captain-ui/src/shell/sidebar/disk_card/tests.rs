use super::used_of;

const MB: u64 = 1 << 20;
const GB: u64 = 1 << 30;

#[test]
fn the_unit_shows_once_when_both_amounts_share_it() {
    assert_eq!(used_of(18 * GB, Some(64 * GB)), "18.0 of 64.0 GB");
    assert_eq!(used_of(500 * MB, Some(64 * GB)), "500 MB of 64.0 GB");
    assert_eq!(used_of(18 * GB, None), "18.0 GB");
}
