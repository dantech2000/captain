use std::time::{Duration, Instant};

use super::{HISTORY_LEN, StatsHistory};
use crate::model::StatsSample;

fn sample(cpu: f64, rx: u64, tx: u64) -> StatsSample {
    StatsSample {
        cpu_percent: cpu,
        memory_bytes: 1024,
        rx_bytes: rx,
        tx_bytes: tx,
    }
}

#[test]
fn keeps_only_the_latest_samples() {
    let mut history = StatsHistory::default();
    for i in 0..HISTORY_LEN + 5 {
        history.push(sample(i as f64, 0, 0));
    }
    let cpu = history.cpu_series();
    assert_eq!(cpu.len(), HISTORY_LEN);
    assert_eq!(cpu[0], 5.0);
    assert_eq!(
        history.latest().map(|s| s.cpu_percent),
        Some((HISTORY_LEN + 4) as f64)
    );
}

#[test]
fn net_rate_divides_the_delta_by_the_time_between_samples() {
    let start = Instant::now();
    let mut history = StatsHistory::default();
    history.push_at(sample(0.0, 1000, 500), start);
    history.push_at(sample(0.0, 1800, 700), start + Duration::from_secs(1));
    assert_eq!(history.net_rate(), (800, 200));
    // After a 30 s gap, 30 000 bytes are 1 000 bytes per second.
    history.push_at(sample(0.0, 31_800, 700), start + Duration::from_secs(31));
    assert_eq!(history.net_rate(), (1000, 0));
    assert_eq!(history.net_series().len(), 2);
}

#[test]
fn net_rate_survives_a_counter_reset() {
    let mut history = StatsHistory::default();
    history.push(sample(0.0, 1000, 1000));
    history.push(sample(0.0, 10, 10));
    assert_eq!(history.net_rate(), (0, 0));
}
