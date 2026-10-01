use super::backend_extension;

#[test]
fn the_label_or_the_project_prefix_marks_an_extension_backend() {
    let from_label = backend_extension(Some("acme-1234abcd"), Some("other"));
    assert_eq!(from_label.as_deref(), Some("acme-1234abcd"));
    let from_project = backend_extension(None, Some("captain-ext-acme-1234abcd"));
    assert_eq!(from_project.as_deref(), Some("acme-1234abcd"));
    assert_eq!(backend_extension(None, Some("captain-web")), None);
    assert_eq!(backend_extension(None, None), None);
}
