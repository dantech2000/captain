use std::collections::HashMap;

use super::StatsHistory;
use crate::model::StatsSample;

/// Stats histories for every container Captain follows, keyed by container ID.
#[derive(Debug, Clone, Default)]
pub struct StatsBoard {
    histories: HashMap<String, StatsHistory>,
}

impl StatsBoard {
    pub fn push(&mut self, id: &str, sample: StatsSample) {
        self.histories
            .entry(id.to_string())
            .or_default()
            .push(sample);
    }

    pub fn get(&self, id: &str) -> Option<&StatsHistory> {
        self.histories.get(id)
    }

    /// Drops histories for containers that `keep` rejects.
    pub fn retain(&mut self, keep: impl Fn(&str) -> bool) {
        self.histories.retain(|id, _| keep(id));
    }

    /// A board with only the containers that `keep` accepts, for totals over part of
    /// the list.
    pub fn only(&self, keep: impl Fn(&str) -> bool) -> StatsBoard {
        let mut board = self.clone();
        board.retain(keep);
        board
    }

    /// When the newest sample of any container arrived.
    pub fn last_at(&self) -> Option<std::time::Instant> {
        self.histories
            .values()
            .filter_map(StatsHistory::last_at)
            .max()
    }

    pub fn total_cpu(&self) -> f64 {
        self.latest().map(|s| s.cpu_percent).sum()
    }

    pub fn total_memory(&self) -> u64 {
        self.latest().map(|s| s.memory_bytes).sum()
    }

    /// Received plus sent bytes per second, over all containers.
    pub fn total_net_rate(&self) -> u64 {
        self.histories
            .values()
            .map(|h| {
                let (rx, tx) = h.net_rate();
                rx + tx
            })
            .sum()
    }

    pub fn total_cpu_series(&self) -> Vec<f64> {
        sum_aligned(self.histories.values().map(StatsHistory::cpu_series))
    }

    pub fn total_memory_series(&self) -> Vec<f64> {
        sum_aligned(self.histories.values().map(StatsHistory::memory_series))
    }

    pub fn total_net_series(&self) -> Vec<f64> {
        sum_aligned(self.histories.values().map(|h| {
            h.net_series()
                .into_iter()
                .map(|(rx, tx)| (rx + tx) as f64)
                .collect()
        }))
    }

    fn latest(&self) -> impl Iterator<Item = &StatsSample> {
        self.histories.values().filter_map(StatsHistory::latest)
    }
}

/// Adds series together, lining them up at the newest end.
fn sum_aligned(series: impl Iterator<Item = Vec<f64>>) -> Vec<f64> {
    let mut total: Vec<f64> = Vec::new();
    for values in series {
        if values.len() > total.len() {
            let mut padded = vec![0.0; values.len() - total.len()];
            padded.append(&mut total);
            total = padded;
        }
        let offset = total.len() - values.len();
        for (slot, value) in total[offset..].iter_mut().zip(values) {
            *slot += value;
        }
    }
    total
}

#[cfg(test)]
mod tests;
