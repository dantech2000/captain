use super::*;
use crate::new_project::ComposeDoc;

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| item.to_string()).collect()
}

#[test]
fn the_readme_commands_convert() {
    let run = convert_docker_run(
        "docker run --name some-postgres -e POSTGRES_PASSWORD=mysecretpassword -d postgres",
    )
    .unwrap();
    assert_eq!(run.service_name, "some-postgres");
    assert_eq!(run.container_name.as_deref(), Some("some-postgres"));
    assert_eq!(run.service.image, "postgres");
    assert_eq!(
        run.service.environment,
        [(
            "POSTGRES_PASSWORD".to_string(),
            Some("mysecretpassword".to_string())
        )]
    );
    assert!(run.warnings.is_empty());

    let run = convert_docker_run(
        "docker run --name some-redis -d redis redis-server --save 60 1 --loglevel warning",
    )
    .unwrap();
    assert_eq!(run.service.image, "redis");
    assert_eq!(
        run.service.command,
        strings(&["redis-server", "--save", "60", "1", "--loglevel", "warning"])
    );

    let run = convert_docker_run(
        "$ docker run --name some-nginx -v /some/content:/usr/share/nginx/html:ro -d -p 8080:80 nginx",
    )
    .unwrap();
    assert_eq!(run.service.image, "nginx");
    assert_eq!(
        run.service.volumes,
        strings(&["/some/content:/usr/share/nginx/html:ro"])
    );
}

const LONG: &str = r#"docker container run -it --rm \
  -p 127.0.0.1:8080:80 -p8443:443/tcp \
  -e "GREETING=hello world" --env=DEBUG=1 -e HOME \
  -v app-data:/var/lib/app \
  --restart unless-stopped --network mynet \
  --privileged --cap-add NET_ADMIN --memory 512m --cpus 1.5 \
  --label com.example.team=web -l=traefik.enable \
  --entrypoint /bin/sh -w /srv -u 1000:1000 --hostname api --platform linux/amd64 \
  ghcr.io/acme/api:1.2 -c "echo ready""#;

#[test]
fn a_long_command_converts_every_mapped_flag_and_warns_about_the_rest() {
    let run = convert_docker_run(LONG).unwrap();
    let s = &run.service;

    assert_eq!(run.service_name, "api");
    assert_eq!(s.image, "ghcr.io/acme/api:1.2");
    assert_eq!(s.ports, strings(&["127.0.0.1:8080:80", "8443:443/tcp"]));
    assert_eq!(
        s.environment,
        [
            ("GREETING".to_string(), Some("hello world".to_string())),
            ("DEBUG".to_string(), Some("1".to_string())),
            ("HOME".to_string(), None),
        ]
    );
    assert_eq!(s.volumes, strings(&["app-data:/var/lib/app"]));
    assert_eq!(s.restart.as_deref(), Some("unless-stopped"));
    assert_eq!(s.networks, strings(&["mynet"]));
    assert!(s.stdin_open && s.tty);
    assert_eq!(s.mem_limit.as_deref(), Some("512m"));
    assert_eq!(s.cpus.as_deref(), Some("1.5"));
    assert_eq!(
        s.labels,
        [
            ("com.example.team".to_string(), "web".to_string()),
            ("traefik.enable".to_string(), String::new()),
        ]
    );
    assert_eq!(s.entrypoint, strings(&["/bin/sh"]));
    assert_eq!(s.working_dir.as_deref(), Some("/srv"));
    assert_eq!(s.user.as_deref(), Some("1000:1000"));
    assert_eq!(s.hostname.as_deref(), Some("api"));
    assert_eq!(s.platform.as_deref(), Some("linux/amd64"));
    assert_eq!(s.command, strings(&["-c", "echo ready"]));
    assert_eq!(run.warnings.len(), 4, "{:?}", run.warnings);
    assert!(run.warnings[0].contains("--rm"));
    assert!(run.warnings[1].starts_with("HOME has no value"));
    assert!(run.warnings[2].contains("--privileged"));
    assert!(run.warnings[3].contains("--cap-add"));
}

#[test]
fn special_networks_become_network_mode() {
    let run = convert_docker_run("docker run --net=host --restart=no busybox").unwrap();

    assert_eq!(run.service.network_mode.as_deref(), Some("host"));
    assert!(run.service.networks.is_empty());
    assert_eq!(run.service.restart, None);
}

#[test]
fn commands_that_are_not_docker_run_are_refused() {
    assert!(convert_docker_run("").is_err());
    assert!(convert_docker_run("docker ps -a").is_err());
    assert!(convert_docker_run("docker run -d -p 80:80").is_err());
    assert!(convert_docker_run("docker run -p").is_err());
    // Captain cannot tell whether 50000 is the value of an unknown flag.
    assert!(convert_docker_run("docker run --cpu-quotas 50000 alpine").is_err());
}

#[test]
fn the_conversion_is_a_valid_compose_file() {
    let run = convert_docker_run(LONG).unwrap();
    let text = ComposeDoc {
        name: Some("api".into()),
        services: vec![(run.service_name, run.service)],
    }
    .to_yaml();

    let parsed: serde_json::Value = serde_saphyr::from_str(&text).unwrap();
    assert_eq!(parsed["services"]["api"]["image"], "ghcr.io/acme/api:1.2");
    assert!(parsed["volumes"]["app-data"].is_object(), "{text}");
    assert_eq!(parsed["networks"]["mynet"]["external"], true, "{text}");
    assert_eq!(parsed["services"]["api"]["cpus"], "1.5");
}

#[test]
fn a_flag_that_takes_a_value_keeps_it_from_the_image() {
    let run = convert_docker_run("docker run --cpu-quota 50000 alpine").unwrap();

    assert_eq!(run.service.image, "alpine");
    assert!(run.warnings[0].contains("--cpu-quota"));
}

#[test]
fn relative_folders_get_a_warning() {
    let run = convert_docker_run(
        "docker run --env-file ./app.env -v ./site:/srv -v data:/data -v /abs:/abs nginx",
    )
    .unwrap();

    assert_eq!(run.warnings.len(), 2, "{:?}", run.warnings);
    assert!(run.warnings[0].starts_with("./app.env in --env-file"));
    assert!(run.warnings[1].starts_with("./site in -v"));
}
