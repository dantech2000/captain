use super::EnvVar;

#[test]
fn parses_key_and_value() {
    let var = EnvVar::parse("DATABASE_URL=postgres://db/shop?a=b");
    assert_eq!(var.key, "DATABASE_URL");
    assert_eq!(var.value, "postgres://db/shop?a=b");
}

#[test]
fn entry_without_equals_has_empty_value() {
    assert_eq!(EnvVar::parse("DEBUG").value, "");
}

#[test]
fn secrets_are_masked() {
    for key in [
        "POSTGRES_PASSWORD",
        "stripe_secret",
        "GITHUB_TOKEN",
        "OPENAI_API_KEY",
    ] {
        let var = EnvVar::parse(&format!("{key}=hunter2"));
        assert!(var.is_secret(), "{key}");
        assert_eq!(var.display_value(), "••••••••");
    }
}

#[test]
fn plain_values_are_shown() {
    let var = EnvVar::parse("NODE_ENV=production");
    assert!(!var.is_secret());
    assert_eq!(var.display_value(), "production");
}
