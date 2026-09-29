use super::PullTracker;
use crate::model::PullProgress;

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-4
}

#[test]
fn starts_empty() {
    let tracker = PullTracker::new("busybox:latest");
    assert_eq!(tracker.reference(), "busybox:latest");
    assert_eq!(tracker.fraction(), 0.0);
    assert_eq!(tracker.layer_counts(), (0, 0));
}

#[test]
fn status_messages_update_the_status_line() {
    let mut tracker = PullTracker::new("nginx:latest");
    tracker.apply(&PullProgress {
        layer: Some("latest".into()),
        status: "Pulling from library/nginx".into(),
        ..PullProgress::default()
    });
    assert_eq!(tracker.status(), "Pulling from library/nginx");
    assert_eq!(tracker.layer_counts(), (0, 0));

    tracker.apply(&PullProgress::status("Digest: sha256:abc"));
    assert_eq!(tracker.status(), "Digest: sha256:abc");
}

#[test]
fn download_and_extract_fill_one_layer() {
    let mut tracker = PullTracker::new("a");
    tracker.apply(&PullProgress::layer("l1", "Pulling fs layer", None));
    assert_eq!(tracker.fraction(), 0.0);

    tracker.apply(&PullProgress::layer("l1", "Downloading", Some((50, 100))));
    assert!(close(tracker.fraction(), 0.4));

    tracker.apply(&PullProgress::layer("l1", "Download complete", None));
    assert!(close(tracker.fraction(), 0.8));

    tracker.apply(&PullProgress::layer("l1", "Extracting", Some((50, 100))));
    assert!(close(tracker.fraction(), 0.9));

    tracker.apply(&PullProgress::layer("l1", "Pull complete", None));
    assert!(close(tracker.fraction(), 1.0));
    assert_eq!(tracker.layer_counts(), (1, 1));
}

#[test]
fn layers_are_averaged() {
    let mut tracker = PullTracker::new("a");
    tracker.apply(&PullProgress::layer("l1", "Already exists", None));
    tracker.apply(&PullProgress::layer("l2", "Waiting", None));
    tracker.apply(&PullProgress::layer("l3", "Downloading", Some((25, 100))));
    // (1.0 + 0.0 + 0.2) / 3
    assert!(close(tracker.fraction(), 0.4));
    assert_eq!(tracker.layer_counts(), (1, 3));
}

#[test]
fn a_layer_never_moves_backwards() {
    let mut tracker = PullTracker::new("a");
    tracker.apply(&PullProgress::layer("l1", "Download complete", None));
    tracker.apply(&PullProgress::layer("l1", "Downloading", Some((10, 100))));
    assert!(close(tracker.fraction(), 0.8));
}

#[test]
fn zero_totals_do_not_divide_by_zero() {
    let mut tracker = PullTracker::new("a");
    tracker.apply(&PullProgress::layer("l1", "Downloading", Some((0, 0))));
    assert_eq!(tracker.fraction(), 0.0);
}

#[test]
fn finish_fills_every_layer() {
    let mut tracker = PullTracker::new("a");
    tracker.apply(&PullProgress::layer("l1", "Pull complete", None));
    tracker.apply(&PullProgress::layer("config", "Download complete", None));
    assert!(tracker.fraction() < 1.0);
    assert!(!tracker.is_finished());

    tracker.finish();
    assert!(tracker.is_finished());
    assert_eq!(tracker.fraction(), 1.0);
    assert_eq!(tracker.layer_counts(), (2, 2));
}
