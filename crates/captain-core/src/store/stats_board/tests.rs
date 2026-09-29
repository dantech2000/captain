use super::{StatsBoard, sum_aligned};
use crate::model::StatsSample;

fn sample(cpu: f64, memory_bytes: u64) -> StatsSample {
    StatsSample {
        cpu_percent: cpu,
        memory_bytes,
        ..StatsSample::default()
    }
}

#[test]
fn totals_use_the_latest_sample_of_each_container() {
    let mut board = StatsBoard::default();
    board.push("a", sample(10.0, 100));
    board.push("a", sample(20.0, 200));
    board.push("b", sample(5.0, 50));

    assert_eq!(board.total_cpu(), 25.0);
    assert_eq!(board.total_memory(), 250);
}

#[test]
fn retain_drops_other_containers() {
    let mut board = StatsBoard::default();
    board.push("a", sample(1.0, 1));
    board.push("b", sample(1.0, 1));
    board.retain(|id| id == "a");

    assert!(board.get("a").is_some());
    assert!(board.get("b").is_none());
}

#[test]
fn sum_aligned_lines_up_the_newest_values() {
    let total = sum_aligned([vec![1.0, 2.0, 3.0], vec![10.0]].into_iter());
    assert_eq!(total, [1.0, 2.0, 13.0]);
}
