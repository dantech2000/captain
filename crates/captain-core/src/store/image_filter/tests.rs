use super::ImageFilter;
use crate::model::Image;

fn image(containers: usize, dangling: bool) -> Image {
    Image {
        id: "sha256:abc".into(),
        containers,
        dangling,
        ..Image::default()
    }
}

#[test]
fn in_use_and_unused_split_on_containers() {
    assert!(ImageFilter::InUse.matches(&image(1, false)));
    assert!(!ImageFilter::InUse.matches(&image(0, false)));
    assert!(ImageFilter::Unused.matches(&image(0, true)));
    assert!(!ImageFilter::Unused.matches(&image(2, false)));
}

#[test]
fn dangling_matches_untagged_images() {
    assert!(ImageFilter::Dangling.matches(&image(0, true)));
    assert!(!ImageFilter::Dangling.matches(&image(0, false)));
}
