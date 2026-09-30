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

#[test]
fn masks_every_secret_in_a_json_line() {
    assert_eq!(
        mask_secrets(r#"{"level":"info","password":"hun\"ter2","token":"abc","port":80}"#),
        r#"{"level":"info","password":"[masked]","token":"[masked]","port":80}"#
    );
    assert_eq!(
        mask_secrets(r#"{"msg":"{\"api_key\":\"abc123\"}"}"#),
        r#"{"msg":"{\"api_key\":\"[masked]\"}"}"#
    );
}

#[test]
fn masks_the_whole_of_a_quoted_value() {
    assert_eq!(
        mask_secrets(r#"password="correct horse battery staple" user=app"#),
        r#"password="[masked]" user=app"#
    );
    assert_eq!(
        mask_secrets("secret = 'two words' done"),
        "secret = '[masked]' done"
    );
}
