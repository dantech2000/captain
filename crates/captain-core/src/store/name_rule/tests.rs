use super::{NameError, validate_name};

#[test]
fn accepts_docker_style_names() {
    for name in ["db", "pg-data", "app_cache.v2", "0abc", "A-1"] {
        assert_eq!(validate_name(name), Ok(()), "{name}");
    }
}

#[test]
fn rejects_empty_and_short_names() {
    assert_eq!(validate_name(""), Err(NameError::Empty));
    assert_eq!(validate_name("a"), Err(NameError::TooShort));
}

#[test]
fn rejects_a_bad_first_character() {
    for name in ["-data", "_data", ".data", " data"] {
        assert_eq!(validate_name(name), Err(NameError::BadStart), "{name}");
    }
}

#[test]
fn rejects_bad_characters_and_names_the_first_one() {
    assert_eq!(validate_name("my data"), Err(NameError::BadChar(' ')));
    assert_eq!(validate_name("a/b:c"), Err(NameError::BadChar('/')));
    assert_eq!(validate_name("café"), Err(NameError::BadChar('é')));
}
