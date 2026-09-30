use super::read;
use crate::settings::{ThemeFamily, doc_link};

/// A wrong value names its key and line, and the other values still load.
#[test]
fn a_wrong_value_names_its_line() {
    let text = "{\n  \"theme\": \"harbor\",\n  \"kubernetes\": {\n    \"port\": 70000\n  }\n}\n";
    let (settings, problems) = read(text).unwrap();
    assert_eq!(settings.theme, ThemeFamily::Harbor);
    assert_eq!(settings.kubernetes.port, 6443);
    let [problem] = problems.as_slice() else {
        panic!("{problems:?}");
    };
    assert_eq!(
        problem.to_string(),
        format!(
            "settings.json line 4: kubernetes.port must be a whole number from 1 to 65535. See {}",
            doc_link("kubernetes.port")
        )
    );
    assert!(problem.link().ends_with("settings.md#kubernetesport"));
}

#[test]
fn unknown_keys_and_padded_text_are_not_problems() {
    let text = r#"{"accent": "purple", "engine_endpoint": " tcp://h:2375 ", "kubernetes": {}}"#;
    assert_eq!(read(text).unwrap().1, []);
}

/// Each field of `engine_resources` is read against its own limit, and the value
/// with a field below it falls back to the default.
#[test]
fn a_resource_below_its_minimum_names_its_line() {
    let text = "{\n  \"engine_resources\": {\n    \"cpus\": 0,\n    \"memory_bytes\": 8589934592,\n    \"disk_bytes\": 0\n  }\n}\n";
    let (settings, problems) = read(text).unwrap();
    assert_eq!(settings.engine_resources, None);
    let lines: Vec<String> = problems
        .iter()
        .map(|problem| {
            problem
                .to_string()
                .split(". See")
                .next()
                .unwrap()
                .to_string()
        })
        .collect();
    assert_eq!(
        lines,
        [
            "settings.json line 3: engine_resources.cpus must be a whole number, 1 or more",
            "settings.json line 5: engine_resources.disk_bytes must be a whole number from 17179869184 to 1099511627776",
        ]
    );
}

#[test]
fn a_nested_field_links_to_its_parents_heading() {
    assert!(doc_link("engine_resources.cpus").ends_with("#engine_resources"));
    assert!(doc_link("kubernetes.port").ends_with("#kubernetesport"));
}
