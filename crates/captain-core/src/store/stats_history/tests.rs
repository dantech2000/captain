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
fn net_rate_is_the_delta_between_the_last_two_samples() {
    let mut history = StatsHistory::default();
    history.push(sample(0.0, 1000, 500));
    history.push(sample(0.0, 1800, 700));
    assert_eq!(history.net_rate(), (800, 200));
    assert_eq!(history.net_series().len(), 1);
}

#[test]
fn net_rate_survives_a_counter_reset() {
    let mut history = StatsHistory::default();
    history.push(sample(0.0, 1000, 1000));
    history.push(sample(0.0, 10, 10));
    assert_eq!(history.net_rate(), (0, 0));
}
