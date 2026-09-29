use super::PortMapping;

fn port(private_port: u16, public_port: Option<u16>, host_ip: Option<&str>) -> PortMapping {
    PortMapping {
        private_port,
        public_port,
        host_ip: host_ip.map(Into::into),
        protocol: "tcp".into(),
    }
}

#[test]
fn unpublished_port_shows_private_side_only() {
    assert_eq!(port(80, None, None).to_string(), "80/tcp");
}

#[test]
fn published_port_shows_host_mapping() {
    assert_eq!(
        port(80, Some(8080), Some("0.0.0.0")).to_string(),
        "0.0.0.0:8080->80/tcp"
    );
}

#[test]
fn ipv6_host_is_bracketed() {
    assert_eq!(
        port(80, Some(8080), Some("::")).to_string(),
        "[::]:8080->80/tcp"
    );
}
