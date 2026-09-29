use super::{MAX_NAME_CHARS, check_name};

#[test]
fn a_name_is_required_trimmed_short_and_unique() {
    let none = || std::iter::empty::<&str>();
    assert!(check_name("before upgrade", none()).is_ok());
    assert!(check_name("", none()).is_err());
    assert!(check_name(" padded", none()).is_err());
    assert!(check_name(&"é".repeat(MAX_NAME_CHARS), none()).is_ok());
    assert!(check_name(&"a".repeat(MAX_NAME_CHARS + 1), none()).is_err());
    assert!(check_name("base", ["other", "base"].into_iter()).is_err());
}
