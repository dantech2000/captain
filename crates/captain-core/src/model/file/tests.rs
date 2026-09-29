use super::*;

#[test]
fn kind_and_mode_label_come_from_the_mode() {
    let folder = FileEntry::new("etc", 0x41ed, 4096, 0);
    assert_eq!(folder.kind, FileKind::Folder);
    assert!(folder.opens);
    assert_eq!(folder.mode_label(), "drwxr-xr-x");
    let link = FileEntry::new("lib64", 0xa1ff, 3, 0);
    assert_eq!(link.kind, FileKind::Link);
    assert!(!link.opens);
    assert_eq!(link.mode_label(), "lrwxrwxrwx");
    // /tmp: sticky and world-writable.
    assert_eq!(
        FileEntry::new("tmp", 0x43ff, 0, 0).mode_label(),
        "drwxrwxrwt"
    );
    assert_eq!(
        FileEntry::new("su", 0o104755, 0, 0).mode_label(),
        "-rwsr-xr-x"
    );
}

#[test]
fn folders_sort_first() {
    let mut entries = vec![
        FileEntry::new("b.txt", 0o100644, 1, 0),
        FileEntry::new("z", 0o040755, 0, 0),
        FileEntry::new("a.txt", 0o100644, 1, 0),
    ];
    sort_entries(&mut entries);
    let names: Vec<_> = entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["z", "a.txt", "b.txt"]);
}
