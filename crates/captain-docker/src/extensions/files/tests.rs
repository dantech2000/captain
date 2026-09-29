use std::path::Path;

use super::unpack;

fn tar_with(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut builder = tar::Builder::new(Vec::new());
    for (path, content) in entries {
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append_data(&mut header, path, *content).unwrap();
    }
    builder.into_inner().unwrap()
}

#[test]
fn a_folder_archive_unpacks_its_contents_only() {
    let dir = std::env::temp_dir().join(format!("captain-unpack-{}", std::process::id()));
    let tar = tar_with(&[("ui/index.html", b"<h1>"), ("ui/js/app.js", b"1")]);
    let written = unpack(&tar, &dir, true).unwrap();
    assert_eq!(written.len(), 2);
    assert_eq!(std::fs::read(dir.join("js/app.js")).unwrap(), b"1");
    assert!(!Path::new(&dir.join("ui")).exists());
    std::fs::remove_dir_all(&dir).ok();
}
