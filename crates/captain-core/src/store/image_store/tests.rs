use super::ImageStore;
use crate::model::Image;
use crate::store::ImageFilter;

fn image(id: &str, tag: Option<&str>, size: u64, created: i64, containers: usize) -> Image {
    Image {
        id: id.into(),
        repo_tags: tag.map(|t| vec![t.to_string()]).unwrap_or_default(),
        size,
        created,
        containers,
        dangling: tag.is_none(),
    }
}

fn store() -> ImageStore {
    let mut store = ImageStore::default();
    store.replace(vec![
        image("a", None, 10, 100, 0),
        image("b", Some("redis:7"), 200, 50, 1),
        image("c", None, 20, 300, 0),
        image("d", Some("alpine:3"), 5, 10, 0),
        image("e", None, 40, 200, 1),
    ]);
    store
}

#[test]
fn new_store_is_empty() {
    let store = ImageStore::default();
    assert!(store.is_empty());
    assert_eq!(store.total_size(), 0);
}

#[test]
fn tagged_images_come_first_by_name_then_untagged_newest_first() {
    let store = store();
    let ids: Vec<_> = store.images().iter().map(|i| i.id.as_str()).collect();
    assert_eq!(ids, ["d", "b", "c", "e", "a"]);
}

#[test]
fn filters_and_counts() {
    let store = store();
    assert_eq!(store.count(ImageFilter::All), 5);
    assert_eq!(store.count(ImageFilter::InUse), 2);
    assert_eq!(store.count(ImageFilter::Unused), 3);
    let dangling: Vec<_> = store
        .filtered(ImageFilter::Dangling)
        .iter()
        .map(|i| i.id.as_str())
        .collect();
    assert_eq!(dangling, ["c", "e", "a"]);
}

#[test]
fn sizes_add_up() {
    let store = store();
    assert_eq!(store.len(), 5);
    assert_eq!(store.total_size(), 275);
    // `e` is dangling but in use, so pruning keeps it.
    assert_eq!(store.dangling_size(), 30);
}

#[test]
fn find_by_id() {
    let store = store();
    assert_eq!(store.find("b").map(|i| i.size), Some(200));
    assert!(store.find("z").is_none());
}
