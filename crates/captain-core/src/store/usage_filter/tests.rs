use super::UsageFilter;
use crate::model::{Network, Volume};

fn volume(containers: Option<usize>) -> Volume {
    Volume {
        containers,
        ..Volume::default()
    }
}

fn network(containers: usize) -> Network {
    Network {
        containers,
        ..Network::default()
    }
}

#[test]
fn in_use_and_unused_split_volumes() {
    assert!(UsageFilter::InUse.matches(&volume(Some(1))));
    assert!(!UsageFilter::InUse.matches(&volume(Some(0))));
    assert!(UsageFilter::Unused.matches(&volume(Some(0))));
    assert!(!UsageFilter::Unused.matches(&volume(Some(2))));
}

#[test]
fn unknown_volume_usage_shows_only_under_all() {
    assert!(!UsageFilter::InUse.matches(&volume(None)));
    assert!(!UsageFilter::Unused.matches(&volume(None)));
}

#[test]
fn in_use_and_unused_split_networks() {
    assert!(UsageFilter::InUse.matches(&network(2)));
    assert!(!UsageFilter::InUse.matches(&network(0)));
    assert!(UsageFilter::Unused.matches(&network(0)));
    assert!(!UsageFilter::Unused.matches(&network(1)));
}
