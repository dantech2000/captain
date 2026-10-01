//! The `docker run` flags Captain knows, from the `docker run` reference
//! (<https://docs.docker.com/reference/cli/docker/container/run/>).

/// What a flag becomes in Compose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Flag {
    Publish,
    Env,
    EnvFile,
    Volume,
    Name,
    Restart,
    Network,
    Workdir,
    Entrypoint,
    User,
    Hostname,
    Label,
    Memory,
    Cpus,
    Platform,
    Pull,
    Interactive,
    Tty,
    Detach,
    Rm,
    /// Not converted, and takes a value.
    OtherValue,
    /// Not converted, and takes none.
    OtherBool,
}

/// Flags Captain does not convert that take a value, so the next word is theirs.
const VALUE_FLAGS: &[&str] = &[
    "mount",
    "cap-add",
    "cap-drop",
    "device",
    "gpus",
    "health-cmd",
    "health-interval",
    "health-retries",
    "health-timeout",
    "health-start-period",
    "health-start-interval",
    "ulimit",
    "log-driver",
    "log-opt",
    "add-host",
    "dns",
    "dns-search",
    "dns-option",
    "shm-size",
    "tmpfs",
    "security-opt",
    "sysctl",
    "ipc",
    "pid",
    "stop-signal",
    "stop-timeout",
    "label-file",
    "link",
    "memory-swap",
    "memory-reservation",
    "cpu-shares",
    "cpuset-cpus",
    "cpuset-mems",
    "expose",
    "runtime",
    "userns",
    "uts",
    "cgroupns",
    "cgroup-parent",
    "isolation",
    "kernel-memory",
    "blkio-weight",
    "device-cgroup-rule",
    "volumes-from",
    "network-alias",
    "ip",
    "ip6",
    "mac-address",
    "domainname",
    "cidfile",
    "annotation",
    "attach",
    "group-add",
    "pids-limit",
    "storage-opt",
    "init-path",
];

impl Flag {
    /// The flag for a long name, without `--`.
    pub(super) fn long(name: &str) -> Flag {
        match name {
            "publish" => Flag::Publish,
            "env" => Flag::Env,
            "env-file" => Flag::EnvFile,
            "volume" => Flag::Volume,
            "name" => Flag::Name,
            "restart" => Flag::Restart,
            "network" | "net" => Flag::Network,
            "workdir" => Flag::Workdir,
            "entrypoint" => Flag::Entrypoint,
            "user" => Flag::User,
            "hostname" => Flag::Hostname,
            "label" => Flag::Label,
            "memory" => Flag::Memory,
            "cpus" => Flag::Cpus,
            "platform" => Flag::Platform,
            "pull" => Flag::Pull,
            "interactive" => Flag::Interactive,
            "tty" => Flag::Tty,
            "detach" => Flag::Detach,
            "rm" => Flag::Rm,
            other if VALUE_FLAGS.contains(&other) => Flag::OtherValue,
            _ => Flag::OtherBool,
        }
    }

    /// The flag for a one-letter name.
    pub(super) fn short(letter: char) -> Flag {
        match letter {
            'p' => Flag::Publish,
            'e' => Flag::Env,
            'v' => Flag::Volume,
            'w' => Flag::Workdir,
            'u' => Flag::User,
            'h' => Flag::Hostname,
            'l' => Flag::Label,
            'm' => Flag::Memory,
            'i' => Flag::Interactive,
            't' => Flag::Tty,
            'd' => Flag::Detach,
            'a' | 'c' => Flag::OtherValue,
            _ => Flag::OtherBool,
        }
    }

    pub(super) fn takes_value(self) -> bool {
        !matches!(
            self,
            Flag::Interactive | Flag::Tty | Flag::Detach | Flag::Rm | Flag::OtherBool
        )
    }
}
