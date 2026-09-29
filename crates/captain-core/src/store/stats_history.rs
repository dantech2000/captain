use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::model::StatsSample;

/// How many samples a history keeps. The engine sends about one a second.
pub const HISTORY_LEN: usize = 60;
/// The shortest time a network rate divides by, so two samples that arrive close
/// together do not show a huge rate.
const MIN_INTERVAL: Duration = Duration::from_millis(250);

/// The recent stats samples of one container, oldest first, with the time each
/// one arrived.
#[derive(Debug, Clone, Default)]
pub struct StatsHistory {
    samples: VecDeque<(Instant, StatsSample)>,
}

impl StatsHistory {
    pub fn push(&mut self, sample: StatsSample) {
        self.push_at(sample, Instant::now());
    }

    /// Records `sample` as arrived at `at`.
    pub fn push_at(&mut self, sample: StatsSample, at: Instant) {
        if self.samples.len() == HISTORY_LEN {
            self.samples.pop_front();
        }
        self.samples.push_back((at, sample));
    }

    pub fn latest(&self) -> Option<&StatsSample> {
        self.samples.back().map(|(_, s)| s)
    }

    pub fn cpu_series(&self) -> Vec<f64> {
        self.samples.iter().map(|(_, s)| s.cpu_percent).collect()
    }

    pub fn memory_series(&self) -> Vec<f64> {
        self.samples
            .iter()
            .map(|(_, s)| s.memory_bytes as f64)
            .collect()
    }

    /// Bytes per second received and sent, between each pair of samples. Each
    /// delta divides by the time between the two samples, so a gap while the
    /// stats stream reconnects shows the average rate, not one burst.
    pub fn net_series(&self) -> Vec<(u64, u64)> {
        self.samples
            .iter()
            .zip(self.samples.iter().skip(1))
            .map(|((at_a, a), (at_b, b))| {
                let seconds = at_b.duration_since(*at_a).max(MIN_INTERVAL).as_secs_f64();
                let rate = |delta: u64| (delta as f64 / seconds).round() as u64;
                (
                    rate(b.rx_bytes.saturating_sub(a.rx_bytes)),
                    rate(b.tx_bytes.saturating_sub(a.tx_bytes)),
                )
            })
            .collect()
    }

    /// The latest received and sent rates, in bytes per second.
    pub fn net_rate(&self) -> (u64, u64) {
        self.net_series().last().copied().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests;
