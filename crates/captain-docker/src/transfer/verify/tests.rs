use super::Measure;

#[test]
fn parses_the_script_output() {
    assert_eq!(
        Measure::parse("12\n4096\n"),
        Some(Measure {
            entries: 12,
            bytes: 4096
        })
    );
    // An exponent form still parses.
    assert_eq!(
        Measure::parse("3\n1.5e+10\n").map(|m| m.bytes),
        Some(15_000_000_000)
    );
    assert_eq!(Measure::parse("12\n"), None);
    assert_eq!(Measure::parse("find: error\n"), None);
}

#[test]
fn check_compares_both_numbers() {
    let a = Measure {
        entries: 2,
        bytes: 10,
    };
    assert!(a.check(&a).is_ok());
    let b = Measure {
        entries: 2,
        bytes: 9,
    };
    assert!(a.check(&b).unwrap_err().contains("10 bytes"));
}
