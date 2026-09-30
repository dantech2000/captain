use serde_json::json;

use super::{ContainerView, NavigateIntent};

#[test]
fn each_sdk_intent_parses_and_an_id_is_required() {
    let parse = NavigateIntent::parse;
    assert_eq!(
        parse("viewContainerLogs", &json!({"id": "abc"})),
        Ok(NavigateIntent::Container {
            id: "abc".into(),
            view: ContainerView::Logs
        })
    );
    assert_eq!(
        parse("viewImage", &json!({"id": "sha256:1", "tag": "1.0"})),
        Ok(NavigateIntent::Image {
            id: "sha256:1".into(),
            tag: "1.0".into()
        })
    );
    assert_eq!(
        parse("viewVolume", &json!({"volume": "data"})),
        Ok(NavigateIntent::Volume("data".into()))
    );
    assert_eq!(
        parse("viewContainers", &json!({})),
        Ok(NavigateIntent::Containers)
    );
    assert!(parse("viewContainer", &json!({})).is_err());
}
