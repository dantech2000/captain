use super::*;

fn doc(service: ServiceSpec) -> ComposeDoc {
    ComposeDoc {
        name: Some("shop".into()),
        services: vec![("web".into(), service)],
    }
}

#[test]
fn a_service_is_written_with_its_volumes_and_networks_declared() {
    let text = doc(ServiceSpec {
        image: "postgres:18".into(),
        restart: Some("unless-stopped".into()),
        command: vec!["postgres".into(), "-c".into(), "max_connections=200".into()],
        environment: vec![
            ("POSTGRES_DB".into(), Some("app".into())),
            ("HOME".into(), None),
        ],
        ports: vec!["5432:5432".into()],
        volumes: vec![
            "pgdata:/var/lib/postgresql".into(),
            "./init:/docker-entrypoint-initdb.d:ro".into(),
        ],
        networks: vec!["backend".into()],
        ..ServiceSpec::default()
    })
    .to_yaml();

    assert_eq!(
        text,
        "name: shop
services:
  web:
    image: postgres:18
    restart: unless-stopped
    command: [\"postgres\", \"-c\", \"max_connections=200\"]
    environment:
      POSTGRES_DB: app
      HOME:
    ports:
      - \"5432:5432\"
    volumes:
      - pgdata:/var/lib/postgresql
      - ./init:/docker-entrypoint-initdb.d:ro
    networks:
      - backend
volumes:
  pgdata: {}
networks:
  backend:
    external: true
"
    );
}

#[test]
fn values_that_yaml_or_compose_would_change_are_quoted_and_dollars_doubled() {
    let text = doc(ServiceSpec {
        image: "redis".into(),
        environment: vec![
            ("FLAG".into(), Some("yes".into())),
            ("COUNT".into(), Some("12".into())),
            ("PASSWORD".into(), Some("a$b \"c\"".into())),
            ("RATIO".into(), Some(".5".into())),
        ],
        ..ServiceSpec::default()
    })
    .to_yaml();

    assert!(text.contains("FLAG: \"yes\"\n"), "{text}");
    assert!(text.contains("COUNT: \"12\"\n"), "{text}");
    assert!(text.contains("PASSWORD: \"a$$b \\\"c\\\"\"\n"), "{text}");
    assert!(text.contains("RATIO: \".5\"\n"), "{text}");
}

#[test]
fn only_names_are_named_volumes() {
    assert_eq!(named_volume("data:/data"), Some("data"));
    assert_eq!(named_volume("./data:/data"), None);
    assert_eq!(named_volume("/srv/data:/data"), None);
    assert_eq!(named_volume("C:\\data:/data"), None);
    assert_eq!(named_volume("/data"), None);
}

#[test]
fn a_name_given_twice_keeps_its_last_value() {
    let text = doc(ServiceSpec {
        image: "alpine".into(),
        environment: vec![
            ("A".into(), Some("one".into())),
            ("B".into(), Some("b".into())),
            ("A".into(), Some("two".into())),
        ],
        ..ServiceSpec::default()
    })
    .to_yaml();

    assert!(
        text.contains("    environment:\n      A: two\n      B: b\n"),
        "{text}"
    );
}
