use captain_core::kubernetes::KubeContexts;

use super::contexts_item;
use crate::tray::menu_model::{TrayCommand, TrayItem};

#[test]
fn checks_the_current_context() {
    let contexts = KubeContexts {
        names: vec!["prod".into(), "captain".into()],
        current: Some("captain".into()),
    };
    let Some(TrayItem::Submenu { label, items, .. }) = contexts_item(&contexts) else {
        panic!("no submenu");
    };
    assert_eq!(label, "Kubernetes Contexts");
    assert_eq!(
        items[1],
        TrayItem::Check {
            label: "captain".into(),
            command: TrayCommand::UseContext("captain".into()),
            checked: true,
        }
    );
    assert!(matches!(&items[0], TrayItem::Check { checked: false, .. }));
}

#[test]
fn no_contexts_means_no_submenu() {
    assert_eq!(contexts_item(&KubeContexts::default()), None);
}
