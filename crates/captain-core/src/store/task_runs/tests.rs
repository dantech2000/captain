use super::TaskRuns;
use crate::model::TaskOutput;

#[test]
fn a_run_that_ends_updates_only_its_own_project() {
    let mut runs = TaskRuns::default();
    let a = runs.start("shop", "migrate").unwrap();
    let b = runs.start("blog", "seed").unwrap();
    assert_eq!(runs.start("blog", "seed"), None);
    let output = TaskOutput {
        exit_code: 0,
        output: "done".into(),
    };
    assert!(runs.finish("shop", a, Some(output.clone())));
    assert!(!runs.finish("blog", a, None));
    let blog = runs.get("blog").unwrap();
    assert_eq!(blog.running, Some((b, "seed".to_string())));
    assert_eq!(blog.last, None);
    let shop = runs.get("shop").unwrap();
    assert_eq!(shop.running, None);
    assert_eq!(shop.last, Some(("migrate".to_string(), output)));
}
