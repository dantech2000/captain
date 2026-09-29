use super::*;

#[test]
fn text_and_binary() {
    let text = FilePreview {
        bytes: b"hello\n".to_vec(),
        size: 6,
    };
    assert_eq!(text.text(), Some("hello\n"));
    let binary = FilePreview {
        bytes: vec![0x7f, b'E', b'L', b'F', 0, 1],
        size: 6,
    };
    assert_eq!(binary.text(), None);
}

#[test]
fn a_truncated_preview_drops_a_cut_character() {
    // "é" is two bytes; the preview holds only the first.
    let preview = FilePreview {
        bytes: vec![b'a', 0xc3],
        size: 3,
    };
    assert!(preview.is_truncated());
    assert_eq!(preview.text(), Some("a"));
}
