use super::{DEFAULT_THROUGHPUT, Estimate, duration_label};

#[test]
fn time_rounds_up() {
    assert_eq!(Estimate::new(0, DEFAULT_THROUGHPUT).seconds, 0);
    assert_eq!(Estimate::new(1, DEFAULT_THROUGHPUT).seconds, 1);
    assert_eq!(
        Estimate::new(DEFAULT_THROUGHPUT * 90, DEFAULT_THROUGHPUT).seconds,
        90
    );
    assert_eq!(Estimate::new(10, 0).seconds, 10);
}

#[test]
fn labels() {
    assert_eq!(duration_label(0), "under a minute");
    assert_eq!(duration_label(59), "under a minute");
    assert_eq!(duration_label(60), "about 1 minute");
    assert_eq!(duration_label(61), "about 2 minutes");
    assert_eq!(duration_label(3600), "about 1 hour");
    assert_eq!(duration_label(4800), "about 1 hour 20 minutes");
    assert_eq!(duration_label(7260), "about 2 hours 1 minute");
}
