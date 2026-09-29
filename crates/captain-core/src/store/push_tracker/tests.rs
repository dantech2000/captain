use super::*;

#[test]
fn layers_count_from_preparing_and_finish_messages() {
    let mut push = PushTracker::new("localhost:5000/app:1");
    for status in [
        "The push refers to repository [localhost:5000/app]",
        "Preparing",
        "Preparing",
        "Preparing",
        "Waiting",
        "Pushing",
        "Pushed",
        "Layer already exists",
    ] {
        push.apply(&PullProgress::status(status));
    }
    assert_eq!(push.layer_counts(), (2, 3));
    assert!((push.fraction() - 2.0 / 3.0).abs() < 1e-6);
    assert_eq!(
        push.status(),
        "The push refers to repository [localhost:5000/app]"
    );

    push.apply(&PullProgress::status("Mounted from library/busybox"));
    push.apply(&PullProgress::status("1: digest: sha256:abc size: 527"));
    assert_eq!(push.layer_counts(), (3, 3));
    assert_eq!(push.status(), "1: digest: sha256:abc size: 527");
    push.finish();
    assert_eq!(push.fraction(), 1.0);
}
