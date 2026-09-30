use captain_core::diagnostics::Fix;
use captain_core::model::{ContainerAction, ContainerState};
use captain_core::problems::Problem;

use crate::tray::dot::Light;
use crate::tray::menu_model::{TrayCommand, TrayItem, build};
use crate::tray::snapshot::{EngineStatus, TraySnapshot};
use crate::tray::test_support::{entry, light, snapshot};

#[test]
fn a_problem_shows_its_line_and_its_fixes() {
    let problem = Problem::OutOfMemory {
        id: "web-id".into(),
        name: "web".into(),
        limit: 256 << 20,
        restarts: 4,
    };
    let menu = build(&TraySnapshot {
        problem: Some(problem.clone()),
        ..snapshot(vec![entry("web", ContainerState::Restarting, None)])
    });
    assert_eq!(
        menu[3..7],
        [
            TrayItem::Status {
                label: problem.line(),
                light: Light::Red,
            },
            TrayItem::command(
                "Raise Memory to 512 MB",
                TrayCommand::RaiseMemory {
                    id: "web-id".into(),
                    name: "web".into(),
                    bytes: 512 << 20,
                }
            ),
            TrayItem::command(
                "Show Logs in a Window",
                TrayCommand::FloatLog {
                    id: "web-id".into(),
                    name: "web".into(),
                }
            ),
            TrayItem::command(
                "Stop web",
                TrayCommand::Container {
                    id: "web-id".into(),
                    action: ContainerAction::Stop,
                }
            ),
        ]
    );

    let failed = Problem::EngineFailed {
        why: "no VM".into(),
        fix: Fix::RestartEngine,
    };
    let menu = build(&TraySnapshot {
        engine: EngineStatus::NeedsAttention,
        problem: Some(failed),
        ..snapshot(Vec::new())
    });
    assert_eq!(light(&menu[0]), Some(Light::Red));
    assert_eq!(
        menu[3],
        TrayItem::command(
            "Restart Captain Engine",
            TrayCommand::RunFix(Fix::RestartEngine)
        )
    );
}
