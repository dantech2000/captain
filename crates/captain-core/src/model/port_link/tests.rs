use super::PortLink;

#[test]
fn opens_web_ports_and_copies_database_ports() {
    assert_eq!(
        PortLink::of(8080, 80),
        PortLink::Open("http://localhost:8080".into())
    );
    assert_eq!(
        PortLink::of(15432, 5432),
        PortLink::Copy {
            address: "localhost:15432".into(),
            service: "Postgres",
        }
    );
}
