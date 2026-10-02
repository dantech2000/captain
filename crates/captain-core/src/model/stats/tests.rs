use super::cpu_percent;

#[test]
fn cpu_percent_is_the_share_of_system_ticks_and_zero_without_progress() {
    // The container used 50 of 400 system ticks on a 4-CPU host: half of one core.
    assert!((cpu_percent(150, 100, 1400, 1000, 4) - 50.0).abs() < 1e-9);
    let zero = [
        // No system progress.
        (150, 100, 1000, 1000),
        // A counter reset.
        (10, 100, 1400, 1000),
        // The first sample, without a previous reading.
        (5000, 0, 90_000, 0),
    ];
    for (cpu, previous_cpu, system, previous_system) in zero {
        assert_eq!(
            cpu_percent(cpu, previous_cpu, system, previous_system, 4),
            0.0
        );
    }
}
