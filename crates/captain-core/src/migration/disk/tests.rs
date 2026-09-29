use super::{DiskCheck, DiskVerdict};

fn verdict(bytes: u64, free: Option<u64>) -> DiskVerdict {
    DiskCheck { bytes, free }.verdict()
}

#[test]
fn needs_twice_the_size() {
    assert_eq!(verdict(100, None), DiskVerdict::Unknown);
    assert_eq!(verdict(100, Some(200)), DiskVerdict::Enough);
    assert_eq!(verdict(100, Some(199)), DiskVerdict::Tight);
    assert_eq!(verdict(100, Some(100)), DiskVerdict::Tight);
    assert_eq!(verdict(100, Some(99)), DiskVerdict::NotEnough);
    assert_eq!(verdict(0, Some(0)), DiskVerdict::Enough);
    assert_eq!(verdict(u64::MAX, Some(u64::MAX)), DiskVerdict::Enough);
}
