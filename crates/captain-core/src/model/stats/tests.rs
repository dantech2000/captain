use super::cpu_percent;

#[test]
fn half_of_one_core() {
    // The container used 50 of 400 system ticks on a 4-CPU host: half of one core.
    assert!((cpu_percent(150, 100, 1400, 1000, 4) - 50.0).abs() < 1e-9);
}

#[test]
fn no_system_progress_reads_zero() {
    assert_eq!(cpu_percent(150, 100, 1000, 1000, 4), 0.0);
}

#[test]
fn counter_reset_reads_zero() {
    assert_eq!(cpu_percent(10, 100, 1400, 1000, 4), 0.0);
}

#[test]
fn first_sample_without_previous_reading_is_zero() {
    assert_eq!(cpu_percent(5000, 0, 90_000, 0, 4), 0.0);
}
