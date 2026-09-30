use super::{MASK, mask_secrets};

#[test]
fn masks_values_after_secret_keys() {
    let cases = [
        (
            "DB_PASSWORD=hunter2 user=app",
            "DB_PASSWORD=[masked] user=app",
        ),
        (
            r#"{"api_token": "abc123", "port": 80}"#,
            r#"{"api_token": "[masked]", "port": 80}"#,
        ),
        ("--api-key=abc123", "--api-key=[masked]"),
        ("password: hunter2", "password: [masked]"),
        (
            "Authorization: Bearer abc.def",
            "Authorization: Bearer [masked]",
        ),
    ];
    for (line, masked) in cases {
        assert_eq!(mask_secrets(line), masked, "{line}");
    }
}

#[test]
fn masks_url_passwords_and_known_tokens() {
    assert_eq!(
        mask_secrets("connecting to postgres://app:s3cret@db:5432/shop"),
        format!("connecting to postgres://app:{MASK}@db:5432/shop")
    );
    assert_eq!(
        mask_secrets("pushed with ghp_0123456789abcdefghij0123"),
        format!("pushed with {MASK}")
    );
    assert_eq!(
        mask_secrets("aws (AKIAIOSFODNN7EXAMPLE)"),
        format!("aws ({MASK})")
    );
}

#[test]
fn leaves_ordinary_lines_alone() {
    let line = "GET /health 200 in 3ms at 12:30:01 from http://web:8080/";
    assert_eq!(mask_secrets(line), line);
}
