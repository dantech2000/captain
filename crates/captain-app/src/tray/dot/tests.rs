use super::Light;

#[test]
fn each_light_has_its_own_circle_in_front_of_the_label() {
    let all = [Light::Green, Light::Amber, Light::Red, Light::Gray];
    let glyphs: std::collections::HashSet<_> = all.iter().map(|l| l.glyph()).collect();
    assert_eq!(glyphs.len(), all.len());
    assert!(Light::Red.label("worker").ends_with(" worker"));
}
