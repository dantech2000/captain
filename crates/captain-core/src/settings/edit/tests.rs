use super::FileEdit;
use crate::settings::overrides::changed;
use crate::settings::{Settings, ThemeFamily};

fn edit(text: &str, before: &Settings, after: &Settings) -> String {
    let mut edit = FileEdit::open(text).unwrap();
    edit.apply(after, &changed(before, after));
    edit.text()
}

#[test]
fn a_comment_survives_a_value_change() {
    let text = r#"{
  // Why this theme.
  "theme": "harbor", /* after */
  "version": 2,

  "kubernetes": {
    // A free port.
    "port": 7000,
  },
}
"#;
    let before = Settings::from_json(text).unwrap();
    let mut after = before.clone();
    after.theme = ThemeFamily::Periwinkle;
    after.kubernetes.port = 7001;
    assert_eq!(
        edit(text, &before, &after),
        text.replace("harbor", "periwinkle").replace("7000", "7001")
    );
}

/// The file holds only what differs from the defaults: a new value adds its key,
/// and a default removes it, with a group it leaves empty.
#[test]
fn only_values_that_differ_from_the_defaults_stay() {
    let text = "{\n  \"version\": 2,\n  \"kubernetes\": {\n    \"port\": 7000\n  }\n}\n";
    let before = Settings::from_json(text).unwrap();
    let mut after = before.clone();
    after.kubernetes.port = 6443;
    after.debug_logging = true;
    assert_eq!(
        edit(text, &before, &after),
        "{\n  \"version\": 2,\n  \"debug_logging\": true\n}\n"
    );
}

#[test]
fn a_new_file_names_the_schema() {
    let after = Settings {
        theme: ThemeFamily::Harbor,
        ..Settings::default()
    };
    let text = edit("", &Settings::default(), &after);
    assert!(
        text.contains("\"$schema\": \"./settings.schema.json\""),
        "{text}"
    );
    assert!(text.contains("\"theme\": \"harbor\""), "{text}");
    assert_eq!(Settings::from_json(&text).unwrap(), after);
}
