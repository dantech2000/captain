use super::*;

#[test]
fn joins_and_finds_parents() {
    assert_eq!(join_path("/", "etc"), "/etc");
    assert_eq!(join_path("/etc", "nginx"), "/etc/nginx");
    assert_eq!(parent_path("/etc/nginx"), Some("/etc".into()));
    assert_eq!(parent_path("/etc"), Some("/".into()));
    assert_eq!(parent_path("/"), None);
}

#[test]
fn breadcrumbs_start_at_the_root() {
    let crumbs = breadcrumbs("/etc/nginx");
    let paths: Vec<_> = crumbs.iter().map(|c| c.path.as_str()).collect();
    assert_eq!(paths, ["/", "/etc", "/etc/nginx"]);
    assert_eq!(crumbs[2].label, "nginx");
}
