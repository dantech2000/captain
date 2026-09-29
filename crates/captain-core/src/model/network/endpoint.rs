/// A container attached to a network, from the network inspect response.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetworkEndpoint {
    pub container_id: String,
    /// The container name without the leading `/`.
    pub name: String,
    /// The IPv4 address with its prefix length, for example `172.18.0.2/16`.
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
    pub mac: Option<String>,
}

impl NetworkEndpoint {
    /// The IPv4 address without the prefix length, or the IPv6 one if there is no IPv4.
    pub fn address(&self) -> Option<&str> {
        self.ipv4
            .as_deref()
            .or(self.ipv6.as_deref())
            .map(|address| address.split('/').next().unwrap_or(address))
    }
}
