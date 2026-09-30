/// What a click on a published port does: open a web page, or copy the address of a
/// service that does not serve web pages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortLink {
    /// Open `http://localhost:PORT`.
    Open(String),
    /// Copy `localhost:PORT`. It holds the service the port usually belongs to.
    Copy {
        address: String,
        service: &'static str,
    },
}

/// Well-known ports of databases, caches, and brokers. They speak their own
/// protocol, so a browser shows an error for them.
const NOT_WEB: &[(u16, &str)] = &[
    (1433, "SQL Server"),
    (1521, "Oracle"),
    (2181, "ZooKeeper"),
    (3306, "MySQL"),
    (4222, "NATS"),
    (5432, "Postgres"),
    (5672, "RabbitMQ"),
    (6379, "Redis"),
    (9042, "Cassandra"),
    (9092, "Kafka"),
    (11211, "Memcached"),
    (27017, "MongoDB"),
];

impl PortLink {
    /// The link for a port published on `localhost`. The container side `private`
    /// decides, because the host side is often a random or shifted number.
    pub fn of(public: u16, private: u16) -> Self {
        let address = format!("localhost:{public}");
        match NOT_WEB.iter().find(|(port, _)| *port == private) {
            Some((_, service)) => Self::Copy { address, service },
            None => Self::Open(format!("http://{address}")),
        }
    }
}

#[cfg(test)]
mod tests;
