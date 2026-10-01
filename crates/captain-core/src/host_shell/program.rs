use std::path::PathBuf;

use crate::cli_tools::login_shell;

/// The shell a new tab runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellProgram {
    pub program: PathBuf,
    pub args: Vec<String>,
}

/// The user's login shell (`$SHELL -l`) on macOS and Linux. On Windows,
/// PowerShell, or `%COMSPEC%` where PowerShell is missing.
pub fn shell_program() -> ShellProgram {
    if cfg!(windows) {
        return windows_shell();
    }
    ShellProgram {
        program: login_shell().unwrap_or_else(|| PathBuf::from("/bin/sh")),
        args: vec!["-l".into()],
    }
}

fn windows_shell() -> ShellProgram {
    let root = std::env::var_os("SystemRoot").map(PathBuf::from);
    let powershell = root
        .map(|root| root.join(r"System32\WindowsPowerShell\v1.0\powershell.exe"))
        .filter(|path| path.is_file());
    match powershell {
        Some(program) => ShellProgram {
            program,
            args: vec!["-NoLogo".into()],
        },
        None => ShellProgram {
            program: std::env::var_os("COMSPEC")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("cmd.exe")),
            args: Vec::new(),
        },
    }
}
