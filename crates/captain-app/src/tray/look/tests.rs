use captain_core::problems::Problem;
use captain_ui::EngineHealth;

use super::{Dot, IconLook, Wheel};
use crate::tray::dot::Light;
use crate::tray::snapshot::TraySnapshot;
use crate::tray::test_support::snapshot;

fn of(engine: EngineHealth, problem: Option<Problem>) -> TraySnapshot {
    TraySnapshot {
        engine,
        problem,
        ..snapshot(Vec::new())
    }
}

#[test]
fn the_dot_is_a_stop_light_for_the_engine_and_its_containers() {
    let restarting = || {
        Some(Problem::Restarting {
            id: "web-id".into(),
            name: "web".into(),
        })
    };
    let unhealthy = || {
        Some(Problem::Unhealthy {
            id: "db-id".into(),
            name: "db".into(),
        })
    };
    // The dot uses the menu's light for the problem: amber unhealthy, red restarting.
    let cases = [
        (of(EngineHealth::Running, None), Some(Light::Green)),
        (of(EngineHealth::Running, unhealthy()), Some(Light::Amber)),
        (of(EngineHealth::Running, restarting()), Some(Light::Red)),
        (of(EngineHealth::Starting, None), Some(Light::Amber)),
        (of(EngineHealth::Reconnecting, None), Some(Light::Amber)),
        (of(EngineHealth::NotAnswering, None), Some(Light::Red)),
        (of(EngineHealth::CannotRun, None), Some(Light::Red)),
        (of(EngineHealth::Stopped, None), None),
    ];
    for (snapshot, light) in &cases {
        assert_eq!(snapshot.icon_light(), *light, "{:?}", snapshot.engine);
    }

    let look = |engine, colored| of(engine, None).look(colored);
    let reconnecting = IconLook {
        wheel: Wheel::Turning,
        dot: Some(Dot::Colored(Light::Amber)),
    };
    assert_eq!(look(EngineHealth::Reconnecting, true), reconnecting);
    // With `menu_bar_status_dot` off, only trouble gets a plain dot.
    assert_eq!(look(EngineHealth::Running, false).dot, None);
    assert_eq!(look(EngineHealth::Reconnecting, false).dot, None);
    assert_eq!(
        look(EngineHealth::NotAnswering, false).dot,
        Some(Dot::Plain)
    );
    assert_eq!(look(EngineHealth::Stopped, true).wheel, Wheel::Dim);
}
