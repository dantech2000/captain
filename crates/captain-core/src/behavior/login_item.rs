//! The text of Captain's login item on each platform: a LaunchAgent plist (macOS), an
//! XDG autostart entry (Linux), and a `Run` key value (Windows). See feature 0015.

/// The LaunchAgent label and the autostart file name. It matches the bundle id.
pub const LOGIN_ITEM_ID: &str = "dev.captain.Captain";

/// The value name under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
pub const RUN_VALUE_NAME: &str = "Captain";

/// The key that holds per-user login programs on Windows.
pub const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

/// A LaunchAgent that starts `program` when the user logs in to the desktop.
pub fn launch_agent_plist(program: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>{LOGIN_ITEM_ID}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{}</string>
  </array>
  <key>RunAtLoad</key><true/>
  <key>ProcessType</key><string>Interactive</string>
  <key>LimitLoadToSessionType</key><string>Aqua</string>
</dict>
</plist>
"#,
        xml_escape(program)
    )
}

/// An XDG autostart entry that starts `program`.
pub fn autostart_entry(program: &str) -> String {
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Captain\n\
         Comment=A native desktop client for Docker\n\
         Exec={}\n\
         Icon={LOGIN_ITEM_ID}\n\
         Terminal=false\n\
         X-GNOME-Autostart-enabled=true\n",
        exec_argument(program)
    )
}

/// True if an autostart entry runs at login. The desktop's startup settings turn
/// an entry off with `Hidden=true` or `X-GNOME-Autostart-enabled=false`.
pub fn autostart_enabled(entry: &str) -> bool {
    !entry.lines().map(str::trim).any(|line| {
        let off = |key: &str, value: &str| {
            line.split_once('=')
                .is_some_and(|(k, v)| k.trim() == key && v.trim().eq_ignore_ascii_case(value))
        };
        off("Hidden", "true") || off("X-GNOME-Autostart-enabled", "false")
    })
}

/// The `Run` value for `program`: its path in double quotes, so spaces are safe.
pub fn run_command(program: &str) -> String {
    format!("\"{program}\"")
}

/// One argument of an `Exec` line, quoted by the Desktop Entry rules: reserved
/// characters need double quotes, and `"`, `` ` ``, `$`, and `\` get a backslash
/// inside them. Then the string escape doubles every backslash and writes line
/// breaks and tabs as `\n`, `\r`, and `\t`, and a literal `%` becomes `%%`, so it
/// is not a field code. See
/// <https://specifications.freedesktop.org/desktop-entry/latest/exec-variables.html>.
fn exec_argument(arg: &str) -> String {
    const RESERVED: &str = " \t\n\"'\\><~|&;$*?#()`";
    let mut quoted = String::new();
    let needs_quotes = arg.chars().any(|c| RESERVED.contains(c));
    if needs_quotes {
        quoted.push('"');
    }
    for c in arg.chars() {
        if needs_quotes && matches!(c, '"' | '`' | '$' | '\\') {
            quoted.push('\\');
        }
        quoted.push(c);
    }
    if needs_quotes {
        quoted.push('"');
    }
    quoted
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .replace('%', "%%")
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests;
