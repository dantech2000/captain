use super::{BridgeRequest, ExecScope, Route, ToastLevel, parse_call};

fn request(message: &str) -> BridgeRequest {
    parse_call(message).unwrap().request
}

#[test]
fn engine_calls_and_window_calls_route_apart() {
    let engine = [
        r#"{"id":1,"method":"extension.vm.service.request","params":{"method":"get","url":"/hello"}}"#,
        r#"{"id":2,"method":"docker.cli.exec","params":{"cmd":"ps","args":[]}}"#,
        r#"{"id":3,"method":"docker.listImages","params":{}}"#,
    ];
    let window = [
        r#"{"id":4,"method":"desktopUI.toast","params":{"level":"error","message":"no"}}"#,
        r#"{"id":5,"method":"host.openExternal","params":{"url":"https://docker.com"}}"#,
        r#"{"id":6,"method":"desktopUI.dialog.showOpenDialog","params":{}}"#,
        r#"{"id":7,"method":"exec.close","params":{"target":2}}"#,
    ];
    assert!(engine.iter().all(|m| request(m).route() == Route::Engine));
    assert!(window.iter().all(|m| request(m).route() == Route::Window));
    assert_eq!(
        request(window[0]),
        BridgeRequest::Toast {
            level: ToastLevel::Error,
            message: "no".into()
        }
    );
}

#[test]
fn an_exec_names_its_scope_and_loses_shell_quotes() {
    let message = r#"{"id":9,"method":"extension.host.cli.exec",
        "params":{"cmd":"tool","args":["--format","\"{{json .}}\"","'a'"],"stream":true}}"#;
    let BridgeRequest::Exec { scope, exec } = request(message) else {
        panic!("not an exec");
    };
    assert_eq!(scope, ExecScope::Host);
    assert_eq!(exec.args, ["--format", "{{json .}}", "a"]);
    assert!(exec.stream);
}

#[test]
fn a_service_call_stays_on_the_backend() {
    let BridgeRequest::Service(service) = request(
        r#"{"id":1,"method":"extension.vm.service.request","params":{"method":"post","url":"/x","data":{"a":1}}}"#,
    ) else {
        panic!("not a service call");
    };
    assert_eq!(service.method, "POST");
    assert_eq!(service.body.as_deref(), Some(r#"{"a":1}"#));
    let absolute = r#"{"id":2,"method":"extension.vm.service.request","params":{"method":"GET","url":"http://evil/"}}"#;
    assert_eq!(parse_call(absolute).unwrap_err().id, Some(2));
}

#[test]
fn an_unknown_method_names_itself() {
    let error = parse_call(r#"{"id":3,"method":"desktopUI.navigate.viewBuilds"}"#).unwrap_err();
    assert_eq!(error.id, Some(3));
    assert_eq!(
        error.message,
        "desktopUI.navigate.viewBuilds is not supported by Captain"
    );
    assert_eq!(parse_call("[").unwrap_err().id, None);
}
