use super::ExposedPort;

fn port(port: u16, protocol: &str) -> ExposedPort {
    ExposedPort {
        port,
        protocol: protocol.into(),
    }
}

#[test]
fn parses_the_engine_form() {
    assert_eq!(ExposedPort::parse("80/tcp"), Some(port(80, "tcp")));
    assert_eq!(ExposedPort::parse("53/UDP"), Some(port(53, "udp")));
    assert_eq!(ExposedPort::parse("8080"), Some(port(8080, "tcp")));
}

#[test]
fn rejects_bad_ports() {
    for text in ["", "0/tcp", "http/tcp", "70000/tcp", "80-90/tcp"] {
        assert_eq!(ExposedPort::parse(text), None, "{text}");
    }
}
