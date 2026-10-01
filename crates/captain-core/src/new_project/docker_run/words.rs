//! Splits a pasted command into words the way a POSIX shell does: quotes,
//! backslashes, and `\` at a line end, which joins the next line.

/// The words of `text`. Fails on a quote with no closing quote.
pub(super) fn split_words(text: &str) -> Result<Vec<String>, String> {
    let mut words = Vec::new();
    let mut word = String::new();
    // True once the word has begun, so `""` gives an empty word.
    let mut in_word = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some('\n') | None => {}
                Some('\r') => {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                }
                Some(next) => {
                    word.push(next);
                    in_word = true;
                }
            },
            '\'' => {
                in_word = true;
                loop {
                    match chars.next() {
                        Some('\'') => break,
                        Some(ch) => word.push(ch),
                        None => return Err(unclosed('\'')),
                    }
                }
            }
            '"' => {
                in_word = true;
                loop {
                    match chars.next() {
                        Some('"') => break,
                        Some('\\') => match chars.next() {
                            Some(next @ ('"' | '\\' | '$' | '`')) => word.push(next),
                            Some('\n') => {}
                            Some('\r') if chars.peek() == Some(&'\n') => {
                                chars.next();
                            }
                            Some(next) => {
                                word.push('\\');
                                word.push(next);
                            }
                            None => return Err(unclosed('"')),
                        },
                        Some(ch) => word.push(ch),
                        None => return Err(unclosed('"')),
                    }
                }
            }
            c if c.is_whitespace() => {
                if in_word {
                    words.push(std::mem::take(&mut word));
                    in_word = false;
                }
            }
            c => {
                word.push(c);
                in_word = true;
            }
        }
    }
    if in_word {
        words.push(word);
    }
    Ok(words)
}

fn unclosed(quote: char) -> String {
    format!("The command has a {quote} that is not closed. Close it and paste it again.")
}

#[cfg(test)]
mod tests;
