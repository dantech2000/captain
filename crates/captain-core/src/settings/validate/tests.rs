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
