use super::parse_df;

#[test]
fn reads_the_available_column() {
    let output = "Filesystem     1024-blocks      Used Available Capacity Mounted on\n\
                  /dev/disk3s5    482797652 459842984  22954668    96%    /System/Volumes/Data\n";
    assert_eq!(parse_df(output), Some(22_954_668 * 1024));
    assert_eq!(parse_df("df: /nope: No such file or directory\n"), None);
}
