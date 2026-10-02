use super::*;
use crate::new_project::ServiceSpec;

#[test]
fn secret_values_go_in_a_private_env_file() {
    let doc = ComposeDoc {
        name: Some("db".into()),
        services: vec![(
            "postgres".into(),
            ServiceSpec {
                image: "postgres".into(),
                environment: vec![
                    ("POSTGRES_PASSWORD".into(), Some("my$ecret".into())),
                    ("POSTGRES_DB".into(), Some("app".into())),
                ],
                ..ServiceSpec::default()
            },
        )],
    };

    let files = project_files(doc).unwrap();

    let compose = &files[0].text;
    assert!(!compose.contains("my$"), "{compose}");
    assert!(
        compose.contains(
            "POSTGRES_PASSWORD: \"${POSTGRES_PASSWORD:?set POSTGRES_PASSWORD in .env}\"\n"
        )
    );
    assert!(compose.contains("POSTGRES_DB: app\n"));
    assert_eq!(files[1].path, ".env");
    assert!(files[1].private);
    assert!(files[1].text.ends_with("\nPOSTGRES_PASSWORD='my$ecret'\n"));
    assert_eq!(files[2], NewFile::new(".gitignore", ".env\n"));
}

#[test]
fn only_the_last_value_of_a_name_can_be_a_secret() {
    for last in [Some(String::new()), None] {
        let doc = ComposeDoc {
            name: None,
            services: vec![(
                "app".into(),
                ServiceSpec {
                    image: "alpine".into(),
                    environment: vec![
                        ("API_TOKEN".into(), Some("old".into())),
                        ("API_TOKEN".into(), last.clone()),
                    ],
                    ..ServiceSpec::default()
                },
            )],
        };

        let files = project_files(doc).unwrap();

        assert_eq!(files.len(), 1, "{last:?}");
        assert!(!files[0].text.contains("old"), "{}", files[0].text);
        assert!(!files[0].text.contains("${API_TOKEN"), "{}", files[0].text);
    }
}
