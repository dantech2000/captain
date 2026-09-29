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
fn displays_like_docker_ps() {
    assert_eq!(port(80, None, None).to_string(), "80/tcp");
    assert_eq!(
        port(80, Some(8080), Some("0.0.0.0")).to_string(),
        "0.0.0.0:8080->80/tcp"
    );
    assert_eq!(
        port(80, Some(8080), Some("::")).to_string(),
        "[::]:8080->80/tcp"
    );
}
