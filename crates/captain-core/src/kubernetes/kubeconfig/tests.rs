use super::{
    captain_config, cluster_ca, contexts, current_context, merge, parse, remove_captain, to_yaml,
};

const K3S_YAML: &str = "apiVersion: v1
clusters:
- cluster:
    certificate-authority-data: Q0E=
    server: https://127.0.0.1:6443
  name: default
contexts:
- context:
    cluster: default
    user: default
  name: default
current-context: default
kind: Config
users:
- name: default
  user:
    client-certificate-data: Q0VSVA==
    client-key-data: S0VZ
";

const USER_YAML: &str = "apiVersion: v1
kind: Config
clusters:
- name: prod
  cluster:
    server: https://prod.example.com
- name: captain
  cluster:
    server: https://127.0.0.1:1
contexts:
- name: prod
  context:
    cluster: prod
    user: prod
    namespace: web
users:
- name: prod
  user:
    token: secret
current-context: prod
";

#[test]
fn renames_k3s_entries_and_points_at_the_port() {
    let config = captain_config(K3S_YAML, 7443).unwrap();
    assert_eq!(config["clusters"][0]["name"], "captain");
    assert_eq!(
        config["clusters"][0]["cluster"]["server"],
        "https://127.0.0.1:7443"
    );
    assert_eq!(
        config["clusters"][0]["cluster"]["certificate-authority-data"],
        "Q0E="
    );
    assert_eq!(cluster_ca(&config).unwrap(), b"CA");
    assert_eq!(config["users"][0]["user"]["client-key-data"], "S0VZ");
    assert_eq!(contexts(&config), ["captain"]);
}

#[test]
fn merge_replaces_only_captain_and_keeps_the_current_context() {
    let captain = captain_config(K3S_YAML, 6443).unwrap();
    let merged = merge(&parse(USER_YAML).unwrap(), &captain);
    let yaml = to_yaml(&merged).unwrap();
    let merged = parse(&yaml).unwrap();
    assert_eq!(contexts(&merged), ["prod", "captain"]);
    assert_eq!(current_context(&merged).as_deref(), Some("prod"));
    let clusters = merged["clusters"].as_array().unwrap();
    assert_eq!(clusters.len(), 2);
    assert_eq!(clusters[1]["cluster"]["server"], "https://127.0.0.1:6443");
    assert_eq!(merged["users"][0]["user"]["token"], "secret");
    assert_eq!(merged["contexts"][0]["context"]["namespace"], "web");
}

#[test]
fn merge_into_an_empty_file_makes_captain_current() {
    let captain = captain_config(K3S_YAML, 6443).unwrap();
    let merged = merge(&parse("").unwrap(), &captain);
    assert_eq!(current_context(&merged).as_deref(), Some("captain"));
    assert_eq!(merged["kind"], "Config");
}

#[test]
fn remove_clears_a_captain_current_context() {
    let captain = captain_config(K3S_YAML, 6443).unwrap();
    let removed = remove_captain(&captain);
    assert!(contexts(&removed).is_empty());
    assert_eq!(current_context(&removed), None);
}
