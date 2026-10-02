use super::*;

fn order() -> Vec<String> {
    ["a", "b", "c", "d"].map(String::from).to_vec()
}

#[test]
fn toggle_adds_and_removes_and_a_plain_click_replaces() {
    let mut selection = MultiSelection::default();
    selection.click("a", SelectMode::Replace, &order());
    selection.click("c", SelectMode::Toggle, &order());
    assert_eq!(selection.keys(), ["a", "c"]);
    selection.click("a", SelectMode::Toggle, &order());
    assert_eq!(selection.keys(), ["c"]);
    // A plain click selects one row.
    selection.click("b", SelectMode::Toggle, &order());
    selection.click("a", SelectMode::Replace, &order());
    assert_eq!(selection.keys(), ["a"]);
}

#[test]
fn shift_selects_from_the_anchor_in_either_direction() {
    let mut selection = MultiSelection::default();
    selection.click("c", SelectMode::Replace, &order());
    selection.click("a", SelectMode::Range, &order());
    assert_eq!(selection.keys(), ["a", "b", "c"]);
    // The anchor stays, so a second Shift-click moves the other end.
    selection.click("d", SelectMode::Range, &order());
    assert_eq!(selection.keys(), ["c", "d"]);
}

#[test]
fn retain_drops_gone_rows_and_their_anchor() {
    let mut selection = MultiSelection::default();
    selection.click("b", SelectMode::Replace, &order());
    selection.click("c", SelectMode::Toggle, &order());
    selection.retain(|key| key != "c");
    assert_eq!(selection.keys(), ["b"]);
    // With no anchor, Shift-click acts like a plain click.
    selection.click("d", SelectMode::Range, &order());
    assert_eq!(selection.keys(), ["d"]);
}
