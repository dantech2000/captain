use super::{DaemonFields, parse_fields};

fn fields() -> DaemonFields<'static> {
    DaemonFields {
        registry_mirrors: "",
        insecure_registries: "",
        custom: "",
        tcp: false,
        tcp_port: "2375",
    }
}

#[test]
fn empty_fields_are_the_defaults() {
    assert_eq!(parse_fields(fields()), Ok(Default::default()));
}

#[test]
fn lists_split_on_lines_and_commas_without_duplicates() {
    let settings = parse_fields(DaemonFields {
        registry_mirrors: "https://a.example\n https://b.example, https://a.example\n",
        insecure_registries: "10.0.0.0/8,registry.local:5000",
        ..fields()
    })
    .unwrap();
    assert_eq!(
        settings.registry_mirrors,
        ["https://a.example", "https://b.example"]
    );
    assert_eq!(
        settings.insecure_registries,
        ["10.0.0.0/8", "registry.local:5000"]
    );
}

#[test]
fn a_mirror_needs_a_scheme_and_an_insecure_registry_must_not_have_one() {
    let mirror = parse_fields(DaemonFields {
        registry_mirrors: "mirror.gcr.io",
        ..fields()
    });
    assert!(mirror.unwrap_err().contains("https://"));
    let insecure = parse_fields(DaemonFields {
        insecure_registries: "http://registry.local:5000",
        ..fields()
    });
    assert!(insecure.unwrap_err().contains("without a scheme"));
}

#[test]
fn custom_must_be_a_json_object_without_managed_keys() {
    let parse = |custom| parse_fields(DaemonFields { custom, ..fields() });
    assert!(
        parse("{\"log-level\": ")
            .unwrap_err()
            .contains("not valid JSON")
    );
    assert!(parse("[1]").unwrap_err().contains("JSON object"));
    assert!(parse("{\"hosts\": []}").unwrap_err().contains("TCP switch"));
    let ok = parse("{\"log-level\": \"warn\"}").unwrap();
    assert_eq!(ok.custom["log-level"], "warn");
}

#[test]
fn port_is_from_1024_to_65535() {
    let parse = |tcp_port| {
        parse_fields(DaemonFields {
            tcp_port,
            ..fields()
        })
    };
    assert_eq!(parse(" 2376 ").unwrap().tcp_port, 2376);
    for bad in ["80", "65536", "abc", ""] {
        assert!(parse(bad).is_err(), "{bad}");
    }
}
