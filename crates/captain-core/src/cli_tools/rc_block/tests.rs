use super::{with_block, without_block};
use crate::cli_tools::Shell;

#[test]
fn a_block_comes_off_as_it_went_on() {
    let block = Shell::Zsh.block();
    for original in ["", "export EDITOR=zed\n", "alias ll='ls -l'"] {
        let added = with_block(original, &block);
        assert!(added.ends_with(&block));
        let expected = match original.ends_with('\n') || original.is_empty() {
            true => original.to_string(),
            false => format!("{original}\n"),
        };
        assert_eq!(without_block(&added), Some(expected));
    }
    assert_eq!(without_block("export EDITOR=zed\n"), None);
}

#[test]
fn an_existing_block_is_replaced_where_it_is() {
    let text = format!("a\n{}b\n", Shell::Zsh.block().replace("$HOME", "/old"));
    let updated = with_block(&text, &Shell::Zsh.block());
    assert_eq!(updated, format!("a\n{}b\n", Shell::Zsh.block()));
    assert_eq!(without_block(&updated).as_deref(), Some("a\nb\n"));
}
