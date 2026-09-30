use super::RestartPolicy;

/// Settings that `docker update` changes on a container in place. `None` leaves a
/// setting as it is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResourceUpdate {
    /// The memory limit in bytes. The swap limit becomes twice that.
    pub memory: Option<u64>,
    /// The CPU limit in billionths of a CPU, like `docker update --cpus`.
    pub nano_cpus: Option<u64>,
    pub restart_policy: Option<RestartPolicy>,
}
