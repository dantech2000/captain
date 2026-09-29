use super::{error_message, parse_version};

#[test]
fn parses_the_version_json() {
    assert_eq!(
        parse_version("{\"version\":\"v5.3.1\"}\n"),
        Some("v5.3.1".into())
    );
    assert_eq!(parse_version("Docker Compose version v2"), None);
    assert_eq!(parse_version("{\"version\":\"\"}"), None);
}

#[test]
fn error_message_prefers_the_error_line() {
    let stderr = " Container shop-web-1  Starting\n\
                  Error response from daemon: port is already allocated\n\
                  \n";
    assert_eq!(
        error_message(stderr),
        Some("Error response from daemon: port is already allocated".into())
    );
}

#[test]
fn error_message_falls_back_to_the_last_line() {
    assert_eq!(
        error_message("no such service: api\n"),
        Some("no such service: api".into())
    );
    assert_eq!(error_message("  \n"), None);
}
