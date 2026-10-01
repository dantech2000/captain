use super::page_key;
use crate::workspace::Page;

#[test]
fn every_rail_page_has_its_own_key() {
    let pages = [
        Page::Containers,
        Page::Images,
        Page::Volumes,
        Page::Networks,
        Page::Snapshots,
        Page::Storage,
        Page::Extensions,
        Page::PortForwarding,
        Page::Diagnostics,
        Page::Settings,
    ];
    let mut keys: Vec<_> = pages.iter().filter_map(|page| page_key(*page)).collect();
    assert_eq!(keys.len(), pages.len());
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(), pages.len());
    assert_eq!(page_key(Page::Settings), Some(","));
}
