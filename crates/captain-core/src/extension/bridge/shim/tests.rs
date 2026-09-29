use super::{SHIM, ShimContext, init_script};
use crate::extension::parse_call;

/// The method names the shim posts: the first argument of `call(`, `exec(`, and the
/// `exec.close` post.
fn shim_methods() -> Vec<&'static str> {
    ["call(\"", "exec(\"", "post(nextId++, \""]
        .iter()
        .flat_map(|prefix| SHIM.split(prefix).skip(1))
        .filter_map(|rest| rest.split('"').next())
        .collect()
}

#[test]
fn captain_knows_every_method_the_shim_posts() {
    let methods = shim_methods();
    assert!(methods.len() >= 10, "found only {methods:?}");
    for method in methods {
        let message = format!(r#"{{"id":1,"method":"{method}","params":{{}}}}"#);
        if let Err(error) = parse_call(&message) {
            assert!(
                !error.message.contains("not supported"),
                "{method}: {error:?}"
            );
        }
    }
}

#[test]
fn the_context_is_filled_in_as_json() {
    let context = ShimContext::for_host("acme".into(), "acme/ext:1".into(), "mac\"book".into());
    let script = init_script(&context);
    assert!(!script.contains("__CAPTAIN_CONTEXT__"));
    assert!(script.contains(r#""hostname":"mac\"book""#));
}
