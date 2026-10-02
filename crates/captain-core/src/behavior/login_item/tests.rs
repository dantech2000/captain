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
fn exec_quotes_and_escapes_only_what_the_desktop_entry_spec_reserves() {
    let cases = [
        // A plain path is not quoted.
        ("/opt/captain/captain", "\nExec=/opt/captain/captain\n"),
        // Inside the quotes `$` and `\` get a backslash; the string escape then
        // doubles every backslash.
        (
            r"/home/me/my apps/$x\captain",
            r#"Exec="/home/me/my apps/\\$x\\\\captain""#,
        ),
        (
            "/home/me/100%/%u\ncaptain",
            r#"Exec="/home/me/100%%/%%u\ncaptain""#,
        ),
    ];
    for (program, exec) in cases {
        let entry = autostart_entry(program);
        assert!(entry.contains(exec), "{entry}");
    }
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
