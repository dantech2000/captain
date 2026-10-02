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
