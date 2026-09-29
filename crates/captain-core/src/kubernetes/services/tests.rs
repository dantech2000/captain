use super::{TargetPort, check_local_port, resolve_target};

#[test]
fn resolves_numbers_names_and_unset_targets() {
    let ports = [(Some("http".to_string()), 8080), (None, 9000)];
    assert_eq!(
        resolve_target(80, &TargetPort::Number(3000), &ports),
        Some(3000)
    );
    assert_eq!(resolve_target(80, &TargetPort::Same, &ports), Some(80));
    assert_eq!(
        resolve_target(80, &TargetPort::Name("http".into()), &ports),
        Some(8080)
    );
    assert_eq!(
        resolve_target(80, &TargetPort::Name("grpc".into()), &ports),
        None
    );
}

#[test]
fn local_ports_must_be_above_1024() {
    assert!(check_local_port(1024).is_err());
    assert!(check_local_port(1025).is_ok());
}
