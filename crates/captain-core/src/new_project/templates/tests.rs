use std::process::Command;

use super::*;

fn values(template: &Template) -> TemplateValues {
    TemplateValues {
        name: format!("captain-agent-{}", template.service),
        ports: (0..template.ports.len())
            .map(|i| 40000 + template.preferred_port(i) % 1000)
            .collect(),
        user: "admin".into(),
        password: "s3cretPassw0rd".into(),
    }
}

#[test]
fn templates_are_yaml_with_the_values_and_no_password() {
    for template in &TEMPLATES {
        let values = values(template);
        let files = template.files(&values);
        let compose = &files[0].text;

        let parsed: serde_json::Value = serde_saphyr::from_str(compose)
            .unwrap_or_else(|error| panic!("{}: {error}", template.title));
        assert_eq!(parsed["name"], values.name.as_str(), "{}", template.title);
        assert!(!compose.contains("{port"), "{}", template.title);
        assert!(!compose.contains(&values.password), "{}", template.title);
        let env = files.iter().find(|file| file.path == ".env");
        assert_eq!(
            env.is_some(),
            template.password.is_some(),
            "{}",
            template.title
        );
        if let Some(env) = env {
            assert!(env.private && env.text.contains(&values.password));
        }
    }
}

/// `docker compose config` accepts each template. Needs the docker CLI with
/// Compose; it does not need an engine.
#[test]
#[ignore = "runs the docker CLI"]
fn compose_accepts_every_template() {
    let base = std::env::temp_dir().join(format!("captain-templates-{}", std::process::id()));
    for template in &TEMPLATES {
        let values = values(template);
        let dir = base.join(&values.name);
        std::fs::remove_dir_all(&dir).ok();
        crate::new_project::write_project(&dir, &template.files(&values)).unwrap();
        let output = Command::new("docker")
            .args(["compose", "config", "--quiet"])
            .current_dir(&dir)
            .output()
            .expect("docker runs");
        assert!(
            output.status.success(),
            "{}: {}",
            template.title,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    std::fs::remove_dir_all(&base).ok();
}
