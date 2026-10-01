use std::time::Duration;

use super::{ParseError, parse};
use crate::grammar::fixture::catalog;
use crate::grammar::{Action, Destination, Target};
use crate::kubernetes::ForwardKey;
use crate::model::{ContainerAction, PortLink, ProjectAction};

fn ok(line: &str) -> Action {
    parse(line, &catalog()).unwrap_or_else(|e| panic!("{line}: {e:?}"))
}

fn error(line: &str) -> String {
    match parse(line, &catalog()) {
        Err(error) => error.message(),
        Ok(action) => panic!("{line} parsed as {action:?}"),
    }
}

fn containers(ids: &[&str], action: ContainerAction) -> Action {
    Action::Containers {
        ids: ids.iter().map(|id| format!("{id}-id")).collect(),
        action,
    }
}

#[test]
fn verbs_run_on_a_container_a_service_or_a_project() {
    assert_eq!(
        ok("restart redis"),
        containers(&["redis"], ContainerAction::Restart)
    );
    assert_eq!(
        ok("stop worker"),
        containers(&["shop-worker-1", "shop-worker-2"], ContainerAction::Stop)
    );
    assert_eq!(
        ok("resume web"),
        containers(&["shop-web-1"], ContainerAction::Unpause)
    );
    assert_eq!(
        ok("restart shop"),
        Action::Project {
            name: "shop".into(),
            action: ProjectAction::Restart
        }
    );
    assert_eq!(
        ok("start shop"),
        Action::Project {
            name: "shop".into(),
            action: ProjectAction::Up
        }
    );
    assert_eq!(ok("pause shop"), {
        let ids = [
            "shop-web-1",
            "shop-api-1",
            "shop-worker-1",
            "shop-worker-2",
            "shop-postgres-1",
        ];
        containers(&ids, ContainerAction::Pause)
    });
}

#[test]
fn up_and_down_take_only_projects() {
    // `blog` is a project and a stopped container; up names only projects.
    assert_eq!(
        ok("down blog"),
        Action::Project {
            name: "blog".into(),
            action: ProjectAction::Down
        }
    );
    assert_eq!(error("up redis"), "No project named \u{201c}redis\u{201d}.");
}

#[test]
fn up_also_names_a_known_project_that_has_no_containers() {
    assert_eq!(
        ok("up notes"),
        Action::Project {
            name: "notes".into(),
            action: ProjectAction::Up
        }
    );
    assert_eq!(
        error("down notes"),
        "No project named \u{201c}notes\u{201d}."
    );
    let names: Vec<String> = crate::grammar::complete("up no", &catalog())
        .into_iter()
        .map(|suggestion| suggestion.line)
        .collect();
    assert_eq!(names, ["up notes"]);
}

#[test]
fn a_service_name_resolves_in_the_current_project_or_by_project_slash_service() {
    assert_eq!(
        ok("restart blog/api"),
        containers(&["blog-api-1"], ContainerAction::Restart)
    );
    let Err(ParseError::Ambiguous { targets, .. }) = parse("restart api", &catalog()) else {
        panic!("api is in two projects");
    };
    assert_eq!(
        targets,
        [
            Target::Container("shop-api-1-id".into()),
            Target::Container("blog-api-1-id".into())
        ]
    );
    let mut current = catalog();
    current.current_project = Some("blog".into());
    assert_eq!(
        parse("restart api", &current),
        Ok(containers(&["blog-api-1"], ContainerAction::Restart))
    );
}

#[test]
fn logs_take_a_time_and_errors() {
    assert_eq!(
        ok("logs web --since 10m --errors"),
        Action::Logs {
            id: "shop-web-1-id".into(),
            since: Some(Duration::from_secs(600)),
            errors: true,
        }
    );
    assert_eq!(
        ok("logs --since=1h web"),
        Action::Logs {
            id: "shop-web-1-id".into(),
            since: Some(Duration::from_secs(3_600)),
            errors: false,
        }
    );
    assert_eq!(ok("logs shop"), Action::ProjectLog("shop".into()));
}

#[test]
fn bad_input_says_what_is_wrong() {
    assert_eq!(
        error("restart nothing"),
        "No container, service, or project named \u{201c}nothing\u{201d}."
    );
    assert_eq!(
        error("restart"),
        "restart needs a container, service, or project."
    );
    assert_eq!(
        error("logs web --since"),
        "--since needs a time, such as 10m, 1h, or 30s."
    );
    assert!(error("logs web --since 10x").contains("is not a time"));
    assert_eq!(
        error("logs web --tail 5"),
        "logs has no option --tail. Use --since or --errors."
    );
    assert_eq!(
        error("restart web --errors"),
        "restart has no options, so --errors does not fit."
    );
    assert!(error("restart web api").starts_with("Too many words."));
    assert_eq!(
        error("logs shop --errors"),
        "The project log has no filters. Name one service, such as shop/web."
    );
    assert_eq!(error("open redis"), "redis publishes no port.");
    assert_eq!(parse("hello", &catalog()), Err(ParseError::NotACommand));
}

#[test]
fn shell_on_a_scaled_service_asks_which_container() {
    let Err(ParseError::Ambiguous { name, targets }) = parse("shell worker", &catalog()) else {
        panic!("worker has two containers");
    };
    assert_eq!(name, "worker");
    assert_eq!(targets.len(), 2);
    assert_eq!(
        ok("shell shop-worker-2"),
        Action::Shell("shop-worker-2-id".into())
    );
}

#[test]
fn forward_names_a_kubernetes_service_and_a_local_port() {
    let key = |namespace: &str, service: &str, port| ForwardKey {
        namespace: namespace.into(),
        service: service.into(),
        port,
    };
    assert_eq!(
        ok("forward svc/default/web 18080"),
        Action::Forward {
            key: key("default", "web", 80),
            local_port: Some(18080)
        }
    );
    assert_eq!(
        ok("forward svc/api:443"),
        Action::Forward {
            key: key("default", "api", 443),
            local_port: None
        }
    );
    assert!(matches!(
        parse("forward svc/web 18080", &catalog()),
        Err(ParseError::Ambiguous { .. })
    ));
    assert_eq!(
        error("forward svc/default/web 80"),
        "Pick a port above 1024."
    );
    let mut off = catalog();
    off.kubernetes = false;
    assert!(matches!(
        parse("forward svc/default/web", &off),
        Err(ParseError::Invalid(_))
    ));
}

#[test]
fn open_and_pages() {
    assert_eq!(
        ok("open web"),
        Action::Open(PortLink::Open("http://localhost:8080".into()))
    );
    assert!(matches!(
        ok("open postgres"),
        Action::Open(PortLink::Copy { .. })
    ));
    assert_eq!(
        ok("open 9000"),
        Action::Open(PortLink::Open("http://localhost:9000".into()))
    );
    assert_eq!(ok("disk"), Action::Go(Destination::Storage));
    assert_eq!(ok("go forwarding"), Action::Go(Destination::PortForwarding));
    assert_eq!(ok("settings"), Action::Go(Destination::Settings));
}
