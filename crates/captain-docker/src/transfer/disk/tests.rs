use super::parse_df;

#[test]
fn reads_the_available_column() {
    let line = "overlay 102687672 43546216 53882192 45% /\n";
    assert_eq!(parse_df(line), Some(53_882_192 * 1024));
    assert_eq!(parse_df("Filesystem 1024-blocks Used"), None);
    assert_eq!(parse_df(""), None);
}
