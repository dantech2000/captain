use serde_json::json;

use super::{BridgeEvent, exec_result, service_result};

#[test]
fn events_become_bridge_calls_with_json_values() {
    let line = BridgeEvent::Output {
        stderr: true,
        line: "it's </script>".into(),
    };
    assert_eq!(
        line.script(4),
        r#"window.__captainBridge&&window.__captainBridge.output(4,{"stderr":"it's </script>"});"#
    );
    assert_eq!(
        BridgeEvent::Exit(1).script(4),
        "window.__captainBridge&&window.__captainBridge.exit(4,1);"
    );
}

#[test]
fn a_failed_command_or_request_rejects() {
    assert!(matches!(
        exec_result("ls", 0, "a".into(), String::new()),
        BridgeEvent::Resolve(_)
    ));
    assert!(matches!(
        exec_result("ls", 2, String::new(), "no".into()),
        BridgeEvent::Reject(_)
    ));
    assert_eq!(
        service_result(200, r#"{"ok":true}"#),
        BridgeEvent::Resolve(json!({"ok": true}))
    );
    assert_eq!(
        service_result(200, "plain"),
        BridgeEvent::Resolve(json!("plain"))
    );
    let BridgeEvent::Reject(error) = service_result(500, "boom") else {
        panic!("not rejected");
    };
    assert_eq!(error["statusCode"], 500);
}
