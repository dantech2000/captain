use super::Image;

fn image(tags: &[&str]) -> Image {
    Image {
        id: "sha256:4a3f0c2d9e8b7a6f5e4d3c2b1a09".into(),
        repo_tags: tags.iter().map(|t| t.to_string()).collect(),
        ..Image::default()
    }
}

#[test]
fn short_id_drops_the_algorithm() {
    assert_eq!(image(&[]).short_id(), "4a3f0c2d9e8b");
    let short = Image {
        id: "abc".into(),
        ..Image::default()
    };
    assert_eq!(short.short_id(), "abc");
}

#[test]
fn display_name_uses_the_first_tag() {
    assert_eq!(
        image(&["nginx:1.27", "nginx:latest"]).display_name(),
        "nginx:1.27"
    );
    assert_eq!(image(&[]).display_name(), "<none>");
}

#[test]
fn repository_and_tag_split_on_the_last_colon() {
    assert_eq!(
        image(&["nginx:1.27"]).repository_and_tag(),
        ("nginx", "1.27")
    );
    assert_eq!(
        image(&["localhost:5000/app:dev"]).repository_and_tag(),
        ("localhost:5000/app", "dev")
    );
    assert_eq!(
        image(&["localhost:5000/app"]).repository_and_tag(),
        ("localhost:5000/app", "")
    );
    assert_eq!(image(&[]).repository_and_tag(), ("<none>", "<none>"));
}

#[test]
fn extra_tags_and_usage() {
    let mut nginx = image(&["nginx:1.27", "nginx:latest"]);
    assert_eq!(nginx.extra_tag_count(), 1);
    assert!(!nginx.in_use());
    nginx.containers = 2;
    assert!(nginx.in_use());
    assert_eq!(image(&[]).extra_tag_count(), 0);
}
