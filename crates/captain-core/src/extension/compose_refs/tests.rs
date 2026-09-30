use super::compose_references;

#[test]
fn the_files_next_to_the_compose_file_are_found_and_outside_paths_left_out() {
    let yaml = r#"
services:
  app:
    image: ${DESKTOP_PLUGIN_IMAGE}
    env_file: [./app.env, {path: conf/extra.env}]
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - ./data:/data
      - portainer_data:/state
      - {type: bind, source: ../escape, target: /x}
configs:
  settings:
    file: ./conf/settings.json
secrets:
  key:
    environment: KEY
volumes:
  portainer_data:
"#;
    assert_eq!(
        compose_references(yaml),
        [
            ".env",
            "app.env",
            "conf/extra.env",
            "conf/settings.json",
            "data"
        ]
    );
}
