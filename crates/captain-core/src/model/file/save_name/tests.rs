use super::*;

#[test]
fn numbers_a_taken_name_before_the_extension() {
    let taken = ["os-release", "nginx.conf", "nginx (1).conf"];
    let exists = |name: &str| taken.contains(&name);
    assert_eq!(save_name("hosts", exists), "hosts");
    assert_eq!(save_name("os-release", exists), "os-release (1)");
    assert_eq!(save_name("nginx.conf", exists), "nginx (2).conf");
}
