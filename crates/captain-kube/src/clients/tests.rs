use tokio::runtime::Builder;

use super::Clients;

fn kubeconfig(namespace: &str) -> String {
    format!(
        "apiVersion: v1\nkind: Config\ncurrent-context: captain\n\
         clusters:\n- name: captain\n  cluster:\n    server: https://127.0.0.1:1\n\
         users:\n- name: captain\n  user: {{}}\n\
         contexts:\n- name: captain\n  context:\n    cluster: captain\n    user: captain\n    namespace: {namespace}\n"
    )
}

#[test]
fn a_changed_kubeconfig_gives_a_new_client() {
    let runtime = Builder::new_current_thread().enable_all().build().unwrap();
    let file = std::env::temp_dir().join(format!("captain-clients-{}.yaml", std::process::id()));
    let clients = Clients::new(file.clone());
    std::fs::write(&file, kubeconfig("before")).unwrap();
    let first = runtime.block_on(clients.get()).unwrap();
    assert_eq!(first.default_namespace(), "before");
    std::fs::write(&file, kubeconfig("after")).unwrap();
    let second = runtime.block_on(clients.get()).unwrap();
    assert_eq!(second.default_namespace(), "after");
    std::fs::remove_file(&file).ok();
}
