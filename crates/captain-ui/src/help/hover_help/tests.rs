use gpui_kit::ElementId;

use super::{Hint, HoverHelp};

fn hint(text: &'static str) -> Hint {
    Hint {
        text: text.into(),
        keys: &[],
    }
}

#[test]
fn leaving_a_control_keeps_the_hint_of_the_next_one() {
    let (a, b) = (ElementId::from("a"), ElementId::from("b"));
    let mut help = HoverHelp::default();
    help.enter(a.clone(), hint("A"));
    help.enter(b.clone(), hint("B"));

    assert!(!help.leave(&a));
    assert_eq!(help.hint().map(|h| h.text.as_ref()), Some("B"));
    assert!(help.leave(&b));
    assert_eq!(help.hint(), None);
}

#[test]
fn leaving_an_inner_control_shows_the_outer_hint_again() {
    let (row, switch) = (ElementId::from("row"), ElementId::from("switch"));
    let mut help = HoverHelp::default();
    help.enter(row.clone(), hint("Row"));
    help.enter(switch.clone(), hint("Switch"));

    assert!(help.leave(&switch));
    assert_eq!(help.hint().map(|h| h.text.as_ref()), Some("Row"));
}

#[test]
fn a_control_that_enters_with_its_card_in_one_move_shows_its_own_hint() {
    let (card, button) = (ElementId::from("card"), ElementId::from("button"));
    let mut help = HoverHelp::default();
    help.begin_batch();
    // GPUI reports the hovers of one move innermost first.
    help.enter(button.clone(), hint("Button"));
    help.enter(card.clone(), hint("Card"));

    assert!(help.end_batch());
    assert_eq!(help.hint().map(|h| h.text.as_ref()), Some("Button"));
    assert!(help.leave(&button));
    assert_eq!(help.hint().map(|h| h.text.as_ref()), Some("Card"));
}

#[test]
fn a_hint_goes_with_its_element() {
    let row = ElementId::from("gone-row");
    let alive = super::super::liveness::Alive::new(row.clone());
    let mut help = HoverHelp::default();
    help.enter(row, hint("node:22-alpine takes 224 MB."));
    assert!(!help.forget_gone());

    drop(alive);
    assert!(help.forget_gone());
    assert_eq!(help.hint(), None);
}
