use futures::StreamExt;
use futures::channel::mpsc;
use futures::executor::block_on;

use super::{ExecInput, ExecSpec};

#[test]
fn the_shell_spec_uses_the_default_shell_with_a_tty() {
    let spec = ExecSpec::shell(120, 40);
    assert!(spec.uses_default_shell());
    assert!(spec.tty);
    assert_eq!((spec.cols, spec.rows), (120, 40));
    assert_eq!(
        spec.env_entries(),
        ["TERM=xterm-256color", "COLORTERM=truecolor"]
    );
}

#[test]
fn input_fails_after_the_session_ends() {
    let (tx, mut rx) = mpsc::unbounded();
    let input = ExecInput::new(tx);
    input.send(b"ls\r".to_vec()).expect("send");
    input.send(Vec::new()).expect("empty send is a no-op");
    assert_eq!(block_on(rx.next()), Some(b"ls\r".to_vec()));
    drop(rx);
    assert!(input.send(b"x".to_vec()).is_err());
}
