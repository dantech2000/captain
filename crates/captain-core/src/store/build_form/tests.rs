use std::path::PathBuf;

use super::*;

fn form() -> BuildForm {
    BuildForm {
        context: "/src/app".into(),
        tag: "myapp".into(),
        ..BuildForm::default()
    }
}

#[test]
fn defaults_fill_the_dockerfile_and_the_tag() {
    let spec = BuildForm {
        build_args: vec!["MSG=hi there".into(), "  ".into(), "EMPTY=".into()],
        target: " builder ".into(),
        ..form()
    }
    .to_spec()
    .unwrap();
    assert_eq!(spec.context, PathBuf::from("/src/app"));
    assert_eq!(spec.dockerfile, PathBuf::from("Dockerfile"));
    assert_eq!(spec.tag, "myapp:latest");
    assert_eq!(
        spec.build_args,
        [
            ("MSG".into(), "hi there".into()),
            ("EMPTY".into(), String::new())
        ]
    );
    assert_eq!(spec.target.as_deref(), Some("builder"));
}

#[test]
fn bad_input_names_the_first_problem() {
    let error = |form: BuildForm| form.to_spec().unwrap_err();
    assert_eq!(
        error(BuildForm {
            context: " ".into(),
            ..form()
        }),
        BuildFormError::NoContext
    );
    assert_eq!(
        error(BuildForm {
            tag: "".into(),
            ..form()
        }),
        BuildFormError::NoTag
    );
    assert_eq!(
        error(BuildForm {
            tag: "my app".into(),
            ..form()
        }),
        BuildFormError::Tag("my app".into())
    );
    assert_eq!(
        error(BuildForm {
            tag: "app@sha256:abc".into(),
            ..form()
        }),
        BuildFormError::Tag("app@sha256:abc".into())
    );
    assert_eq!(
        error(BuildForm {
            build_args: vec!["NOVALUE".into()],
            ..form()
        }),
        BuildFormError::BuildArg("NOVALUE".into())
    );
    assert_eq!(
        error(BuildForm {
            target: "a b".into(),
            ..form()
        }),
        BuildFormError::Target
    );
}
