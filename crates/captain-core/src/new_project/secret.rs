/// Letters and digits only, so a password needs no quoting in `.env`, in YAML, or
/// in a shell.
const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789";

/// A random password of `len` characters from the system's random source.
pub fn random_password(len: usize) -> String {
    let mut out = String::with_capacity(len);
    let mut buf = [0u8; 64];
    // 256 is not a multiple of the alphabet; drop the bytes above the last full
    // round, so each character is as likely as the others.
    let limit = 256 - 256 % ALPHABET.len();
    while out.len() < len {
        getrandom::fill(&mut buf).expect("the system has a random source");
        for byte in buf.iter().map(|b| *b as usize).filter(|b| *b < limit) {
            if out.len() == len {
                break;
            }
            out.push(ALPHABET[byte % ALPHABET.len()] as char);
        }
    }
    out
}

/// Why a password typed in a form cannot go in `.env`, or `None`.
pub fn password_error(password: &str) -> Option<&'static str> {
    if password.len() < 8 {
        return Some("Use at least 8 characters.");
    }
    let plain = password
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~' | '+' | '@'));
    if !plain {
        return Some("Use letters, digits, and - _ . ~ + @ only.");
    }
    None
}

#[cfg(test)]
mod tests;
