use super::snapshot_repo;

#[test]
fn snapshot_names_are_valid_image_names() {
    assert_eq!(snapshot_repo("web"), "captain-migrate/web");
    assert_eq!(snapshot_repo("/My_App.1"), "captain-migrate/my_app.1");
    assert_eq!(snapshot_repo("_odd name-"), "captain-migrate/odd-name");
}
