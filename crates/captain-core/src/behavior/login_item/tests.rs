use super::{autostart_enabled, autostart_entry, launch_agent_plist, run_command};

#[test]
fn plist_escapes_the_program() {
    let plist = launch_agent_plist("/Apps/R&D <dev>/captain");
    assert!(
        plist.contains("<string>/Apps/R&amp;D &lt;dev&gt;/captain</string>"),
        "{plist}"
    );
    assert!(plist.contains("<key>Label</key><string>dev.captain.Captain</string>"));
}

#[test]
fn a_plain_path_is_not_quoted() {
    let entry = autostart_entry("/opt/captain/captain");
    assert!(entry.contains("\nExec=/opt/captain/captain\n"), "{entry}");
}

#[test]
fn exec_quotes_reserved_characters() {
    // Inside the quotes `$` and `\` get a backslash; the string escape then doubles
    // every backslash.
    let entry = autostart_entry(r"/home/me/my apps/$x\captain");
    assert!(
        entry.contains(r#"Exec="/home/me/my apps/\\$x\\\\captain""#),
        "{entry}"
    );
}

#[test]
fn exec_escapes_percent_and_line_breaks() {
    let entry = autostart_entry("/home/me/100%/%u\ncaptain");
    assert!(
        entry.contains(r#"Exec="/home/me/100%%/%%u\ncaptain""#),
        "{entry}"
    );
}

#[test]
fn hidden_or_disabled_entries_are_off() {
    assert!(autostart_enabled(&autostart_entry("/bin/captain")));
    assert!(!autostart_enabled("[Desktop Entry]\nHidden=true\n"));
    assert!(!autostart_enabled(
        "[Desktop Entry]\nX-GNOME-Autostart-enabled=false\n"
    ));
}

#[test]
fn run_command_quotes_the_path() {
    assert_eq!(
        run_command(r"C:\Program Files\Captain\captain.exe"),
        r#""C:\Program Files\Captain\captain.exe""#
    );
}
