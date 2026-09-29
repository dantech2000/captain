use captain_core::model::FileKind;
use tar::{Builder, EntryType, Header};

use super::*;

/// A listing from BusyBox: `lib64` links to the folder `lib`, and a name has a space.
const BUSYBOX: &str = "81ed 0 1790674920 .dockerenv
41ed 12288 1778638909 bin
a1ff 3 1778638909 lib64
81a4 5 1778638909 my notes.txt
---
81ed .dockerenv
41ed bin
41ed lib64
81a4 my notes.txt
";

#[test]
fn stat_listing_reads_modes_and_link_targets() {
    let entries = stat_listing(BUSYBOX);
    assert_eq!(entries.len(), 4);
    let bin = &entries[1];
    assert_eq!(
        (bin.kind, bin.size, bin.modified),
        (FileKind::Folder, 12288, 1778638909)
    );
    let lib64 = &entries[2];
    assert_eq!(lib64.kind, FileKind::Link);
    assert!(lib64.opens);
    assert_eq!(entries[3].name, "my notes.txt");
    assert!(stat_listing("---\n").is_empty());
}

fn tar_of(entries: &[(&str, EntryType, &[u8])]) -> Vec<u8> {
    let mut builder = Builder::new(Vec::new());
    for (path, kind, data) in entries {
        let mut header = Header::new_gnu();
        header.set_entry_type(*kind);
        header.set_mode(0o644);
        header.set_mtime(7);
        header.set_size(data.len() as u64);
        header.set_cksum();
        builder
            .append_data(&mut header, path, *data)
            .expect("append");
    }
    builder.into_inner().expect("tar")
}

#[test]
fn tar_listing_keeps_direct_children() {
    let tar = tar_of(&[
        ("etc/", EntryType::Directory, b""),
        ("etc/hosts", EntryType::Regular, b"127.0.0.1"),
        ("etc/ssl/", EntryType::Directory, b""),
        ("etc/ssl/cert.pem", EntryType::Regular, b"x"),
    ]);
    let entries = tar_listing(&tar).expect("listing");
    let names: Vec<_> = entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["hosts", "ssl"]);
    assert_eq!(entries[0].size, 9);
    assert_eq!(entries[1].mode_label(), "drw-r--r--");
}

#[test]
fn tar_preview_reads_up_to_the_limit() {
    let tar = tar_of(&[("hosts", EntryType::Regular, b"127.0.0.1 localhost")]);
    let preview = tar_preview(&tar, 9).expect("preview");
    assert_eq!(preview.bytes, b"127.0.0.1");
    assert_eq!(preview.size, 19);
}
