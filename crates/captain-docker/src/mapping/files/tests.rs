use captain_core::model::FileKind;
use tar::{Builder, EntryType, Header};

use super::*;

/// A listing from BusyBox: `lib64` links to the folder `lib`, and names hold a
/// space and a newline.
const BUSYBOX: &str = "81ed 0 1790674920
41ed 12288 1778638909
a1ff 3 1778638909
81a4 5 1778638909
---
0.dockerenv\x001bin\x001lib64\x000my\nnotes.txt\0";

#[test]
fn stat_listing_reads_modes_link_targets_and_any_name() {
    let entries = stat_listing(BUSYBOX).expect("listing");
    assert_eq!(entries.len(), 4);
    let bin = &entries[1];
    assert_eq!(
        (bin.kind, bin.size, bin.modified),
        (FileKind::Folder, 12288, 1778638909)
    );
    let lib64 = &entries[2];
    assert_eq!(lib64.kind, FileKind::Link);
    assert!(lib64.opens);
    assert_eq!(entries[3].name, "my\nnotes.txt");
    assert_eq!(stat_listing("---\n"), Some(Vec::new()));
    assert_eq!(stat_listing("81a4 5 1\n---\n"), None);
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
