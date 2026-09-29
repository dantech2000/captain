use super::product;

#[test]
fn names_known_products() {
    assert_eq!(
        product("unix:///Users/me/.rd/docker.sock"),
        Some("Rancher Desktop")
    );
    assert_eq!(
        product("unix:///Users/me/.docker/run/docker.sock"),
        Some("Docker Desktop")
    );
    assert_eq!(product("unix:///var/run/docker.sock"), None);
}
