use super::host_file_name;

#[test]
fn a_container_name_cannot_leave_the_folder_or_name_a_device() {
    assert_eq!(host_file_name("report.txt"), "report.txt");
    assert_eq!(
        host_file_name(r"\..\Desktop\report.txt"),
        "_.._Desktop_report.txt"
    );
    assert_eq!(host_file_name("C:evil"), "C_evil");
    assert_eq!(host_file_name(".."), "download");
    assert_eq!(host_file_name("notes. "), "notes");
    assert_eq!(host_file_name("CON.txt"), "_CON.txt");
    assert_eq!(host_file_name(".hidden"), ".hidden");
}
