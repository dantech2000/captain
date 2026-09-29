use super::quoted;

#[test]
fn quotes_single_quotes_and_refuses_line_breaks() {
    assert_eq!(
        quoted("unix:///Users/o'brien/docker.sock").unwrap(),
        r"'unix:///Users/o'\''brien/docker.sock'"
    );
    assert!(quoted("unix:///a\nrm -rf ~").is_err());
}
