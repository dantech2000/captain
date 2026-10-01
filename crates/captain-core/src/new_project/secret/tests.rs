use super::*;

#[test]
fn passwords_are_random_and_need_no_quoting() {
    let (one, two) = (random_password(24), random_password(24));
    assert_eq!(one.len(), 24);
    assert_ne!(one, two);
    assert_eq!(password_error(&one), None);
    assert!(password_error("short").is_some());
    assert!(password_error("has space in it").is_some());
}
