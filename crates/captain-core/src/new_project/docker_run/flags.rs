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
    /// Not in the `docker run` reference, so Captain cannot tell if the next
    /// word is its value.
    Unknown,
}

/// Flags Captain does not convert that take no value.
const BOOL_FLAGS: &[&str] = &[
    "privileged",
    "init",
    "read-only",
    "publish-all",
    "no-healthcheck",
    "oom-kill-disable",
    "sig-proxy",
    "disable-content-trust",
    "quiet",
    "help",
    "use-api-socket",
];

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
    "blkio-weight-device",
    "cpu-count",
    "cpu-percent",
    "cpu-period",
    "cpu-quota",
    "cpu-rt-period",
    "cpu-rt-runtime",
    "credentialspec",
    "detach-keys",
    "device-read-bps",
    "device-read-iops",
    "device-write-bps",
    "device-write-iops",
    "dns-opt",
    "io-maxbandwidth",
    "io-maxiops",
    "link-local-ip",
    "memory-swappiness",
    "net-alias",
    "oom-score-adj",
    "volume-driver",
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
            other if BOOL_FLAGS.contains(&other) => Flag::OtherBool,
            _ => Flag::Unknown,
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
            'P' | 'q' => Flag::OtherBool,
            _ => Flag::Unknown,
        }
    }

    pub(super) fn takes_value(self) -> bool {
        !matches!(
            self,
            Flag::Interactive | Flag::Tty | Flag::Detach | Flag::Rm | Flag::OtherBool
        )
    }
}
