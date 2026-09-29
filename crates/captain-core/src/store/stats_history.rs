use std::collections::VecDeque;

use crate::model::StatsSample;

/// How many samples a history keeps. The engine sends about one a second.
pub const HISTORY_LEN: usize = 60;

/// The recent stats samples of one container, oldest first.
#[derive(Debug, Clone, Default)]
pub struct StatsHistory {
    samples: VecDeque<StatsSample>,
}

impl StatsHistory {
    pub fn push(&mut self, sample: StatsSample) {
        if self.samples.len() == HISTORY_LEN {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    pub fn latest(&self) -> Option<&StatsSample> {
        self.samples.back()
    }

    pub fn cpu_series(&self) -> Vec<f64> {
        self.samples.iter().map(|s| s.cpu_percent).collect()
    }

    pub fn memory_series(&self) -> Vec<f64> {
        self.samples.iter().map(|s| s.memory_bytes as f64).collect()
    }

    /// Bytes per second received and sent, between each pair of samples.
    /// Samples arrive about once a second, so each delta is one second of traffic.
    pub fn net_series(&self) -> Vec<(u64, u64)> {
        self.samples
            .iter()
            .zip(self.samples.iter().skip(1))
            .map(|(a, b)| {
                (
                    b.rx_bytes.saturating_sub(a.rx_bytes),
                    b.tx_bytes.saturating_sub(a.tx_bytes),
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
