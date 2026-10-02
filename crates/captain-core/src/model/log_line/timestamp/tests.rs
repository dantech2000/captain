use super::{clock, parse, split};

#[test]
fn parses_docker_nanosecond_times_and_zone_offsets() {
    assert_eq!(parse("1970-01-01T00:00:00Z"), Some(0));
    assert_eq!(parse("2024-05-01T12:34:56.123456789Z"), Some(1_714_566_896));
    assert_eq!(parse("2000-02-29T23:59:59.5Z"), Some(951_868_799));
    assert_eq!(parse("2024-05-01T14:34:56+02:00"), Some(1_714_566_896));
    assert_eq!(parse("2024-05-01T07:34:56.1-05:00"), Some(1_714_566_896));
}

#[test]
fn rejects_text_that_is_not_a_time() {
    for text in [
        "",
        "hello world",
        "2024-05-01",
        "2024-05-01 12:34:56Z",
        "2024-13-01T12:34:56Z",
        "2024-05-01T24:00:00Z",
        "2024-05-01T12:34:56",
        "2024-05-01T12:34:56.Z",
        "2024-05-01T12:34:56+0200",
        "2024-05-01T12:34:5xZ",
        "+024-05-01T12:34:56Z",
    ] {
        assert_eq!(parse(text), None, "{text:?}");
    }
}

#[test]
fn splits_the_time_from_the_text() {
    assert_eq!(
        split("2024-05-01T12:34:56.1Z GET / 200"),
        Some((1_714_566_896, "GET / 200"))
    );
    assert_eq!(
        split("2024-05-01T12:34:56Z  indented"),
        Some((1_714_566_896, " indented"))
    );
    assert_eq!(split("2024-05-01T12:34:56Z"), Some((1_714_566_896, "")));
    assert_eq!(split("GET / 200"), None);
}

#[test]
fn formats_a_clock_in_a_zone() {
    let time = 1_714_566_896;
    assert_eq!(clock(time, 0), "12:34:56");
    assert_eq!(clock(time, 2 * 3600), "14:34:56");
    assert_eq!(clock(time, -13 * 3600), "23:34:56");
    assert_eq!(clock(0, -3600), "23:00:00");
}
