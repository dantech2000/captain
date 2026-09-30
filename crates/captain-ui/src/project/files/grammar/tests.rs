use gpui_kit::component::highlighter::{HighlightTheme, SyntaxHighlighter};
use gpui_kit::component::input::Rope;

use super::*;

#[test]
fn the_dockerfile_grammar_highlights_instructions() {
    register_dockerfile();
    let mut highlighter = SyntaxHighlighter::new(DOCKERFILE);
    assert_eq!(highlighter.language().as_ref(), DOCKERFILE);
    let text = "FROM alpine:3.21 AS base\nRUN <<EOF\necho hi\nEOF\nCOPY --from=base /a /b\n";
    highlighter.update(None, &Rope::from(text), None);
    let styles = highlighter.styles(&(0..text.len()), &*HighlightTheme::default_dark());
    assert!(!styles.is_empty());
}
