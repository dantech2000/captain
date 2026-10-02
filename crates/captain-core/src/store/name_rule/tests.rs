use super::{NameError, validate_name};

#[test]
fn names_follow_docker_rules_and_the_error_names_the_first_problem() {
    for name in ["db", "pg-data", "app_cache.v2", "0abc", "A-1"] {
        assert_eq!(validate_name(name), Ok(()), "{name}");
    }
    let cases = [
        ("", NameError::Empty),
        ("a", NameError::TooShort),
        ("-data", NameError::BadStart),
        ("_data", NameError::BadStart),
        (".data", NameError::BadStart),
        (" data", NameError::BadStart),
        ("my data", NameError::BadChar(' ')),
        ("a/b:c", NameError::BadChar('/')),
        ("café", NameError::BadChar('é')),
    ];
    for (name, error) in cases {
        assert_eq!(validate_name(name), Err(error), "{name:?}");
    }
}
