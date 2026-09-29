/// Names Windows keeps for devices, with or without an extension.
const RESERVED: [&str; 22] = [
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

/// A container file name that is safe as one file name on the host, on every
/// platform. Path separators, `:`, the other characters Windows refuses, and control
/// characters become `_`. Trailing dots and spaces go, since Windows drops them, so
/// `..` is not a parent. A Windows device name such as `CON.txt` gets a leading `_`.
/// A name with nothing left is `download`. See
/// <https://learn.microsoft.com/windows/win32/fileio/naming-a-file>.
pub fn host_file_name(name: &str) -> String {
    let replaced: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    let name = replaced.trim_end_matches(['.', ' ']);
    if name.is_empty() {
        return "download".into();
    }
    let stem = name.split('.').next().unwrap_or_default().trim_end();
    if RESERVED.contains(&stem.to_ascii_lowercase().as_str()) {
        return format!("_{name}");
    }
    name.to_string()
}

#[cfg(test)]
mod tests;
