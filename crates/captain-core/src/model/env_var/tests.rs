use super::EnvVar;

#[test]
fn parses_key_and_value() {
    let var = EnvVar::parse("DATABASE_URL=postgres://db/shop?a=b");
    assert_eq!(var.key, "DATABASE_URL");
    assert_eq!(var.value, "postgres://db/shop?a=b");
    assert_eq!(EnvVar::parse("DEBUG").value, "");
}

#[test]
fn only_secrets_are_masked() {
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
    let plain = EnvVar::parse("NODE_ENV=production");
    assert!(!plain.is_secret());
    assert_eq!(plain.display_value(), "production");
}
