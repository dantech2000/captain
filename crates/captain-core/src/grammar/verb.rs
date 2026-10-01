/// The first word of a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verb {
    Start,
    Stop,
    Restart,
    Pause,
    Resume,
    Up,
    Down,
    Logs,
    Shell,
    Open,
    Forward,
    Float,
    Disk,
    Go,
}

/// What a word after the verb may name.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct Kinds {
    pub container: bool,
    pub service: bool,
    pub project: bool,
    /// A known project that has no containers, which only `up` can start.
    pub stopped: bool,
    pub port: bool,
    pub kube: bool,
    pub page: bool,
}

const RUNNABLE: Kinds = Kinds {
    container: true,
    service: true,
    project: true,
    stopped: false,
    port: false,
    kube: false,
    page: false,
};
const PROJECT: Kinds = Kinds {
    project: true,
    ..NONE
};
const UP: Kinds = Kinds {
    stopped: true,
    ..PROJECT
};
const CONTAINER: Kinds = Kinds {
    container: true,
    service: true,
    ..NONE
};
const OPENER: Kinds = Kinds {
    port: true,
    ..CONTAINER
};
const KUBE: Kinds = Kinds { kube: true, ..NONE };
const PORT: Kinds = Kinds { port: true, ..NONE };
const PAGE: Kinds = Kinds { page: true, ..NONE };
const NONE: Kinds = Kinds {
    container: false,
    service: false,
    project: false,
    stopped: false,
    port: false,
    kube: false,
    page: false,
};

impl Kinds {
    /// The nouns for a message, for example "container, service, or project".
    pub fn nouns(self) -> String {
        let nouns: Vec<&str> = [
            (self.container, "container"),
            (self.service, "service"),
            (self.project, "project"),
            (self.kube, "Kubernetes service"),
            (self.page, "page"),
            (self.port, "port"),
        ]
        .into_iter()
        .filter_map(|(on, noun)| on.then_some(noun))
        .collect();
        match nouns.as_slice() {
            [] => String::new(),
            [one] => (*one).to_string(),
            [a, b] => format!("{a} or {b}"),
            [rest @ .., last] => format!("{}, or {last}", rest.join(", ")),
        }
    }
}

/// An option of `logs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    Since,
    Errors,
}

impl Flag {
    pub const ALL: [Flag; 2] = [Flag::Since, Flag::Errors];

    /// The flag as typed, for example `--since`.
    pub fn text(self) -> &'static str {
        match self {
            Flag::Since => "--since",
            Flag::Errors => "--errors",
        }
    }

    pub fn help(self) -> &'static str {
        match self {
            Flag::Since => "Show only lines from a time on, such as --since 10m.",
            Flag::Errors => "Show only error lines.",
        }
    }
}

impl Verb {
    pub const ALL: [Verb; 14] = [
        Verb::Start,
        Verb::Stop,
        Verb::Restart,
        Verb::Pause,
        Verb::Resume,
        Verb::Up,
        Verb::Down,
        Verb::Logs,
        Verb::Shell,
        Verb::Open,
        Verb::Forward,
        Verb::Float,
        Verb::Disk,
        Verb::Go,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Verb::Start => "start",
            Verb::Stop => "stop",
            Verb::Restart => "restart",
            Verb::Pause => "pause",
            Verb::Resume => "resume",
            Verb::Up => "up",
            Verb::Down => "down",
            Verb::Logs => "logs",
            Verb::Shell => "shell",
            Verb::Open => "open",
            Verb::Forward => "forward",
            Verb::Float => "float",
            Verb::Disk => "disk",
            Verb::Go => "go",
        }
    }

    /// The verb whose name is `word`.
    pub fn parse(word: &str) -> Option<Verb> {
        Verb::ALL.into_iter().find(|verb| verb.name() == word)
    }

    /// How to write the command, for example `logs <container|service> [--since 10m] [--errors]`.
    pub fn usage(self) -> &'static str {
        match self {
            Verb::Start => "start <container|service|project>",
            Verb::Stop => "stop <container|service|project>",
            Verb::Restart => "restart <container|service|project>",
            Verb::Pause => "pause <container|service|project>",
            Verb::Resume => "resume <container|service|project>",
            Verb::Up => "up <project>",
            Verb::Down => "down <project>",
            Verb::Logs => "logs <container|service|project> [--since 10m] [--errors]",
            Verb::Shell => "shell <container|service>",
            Verb::Open => "open <container|service|port>",
            Verb::Forward => "forward svc/<name> [local port]",
            Verb::Float => "float <container|service>",
            Verb::Disk => "disk",
            Verb::Go => "go <page>",
        }
    }

    /// The status bar sentence for the verb before it has a name.
    pub fn help(self) -> &'static str {
        match self {
            Verb::Start => "Start a container, a service, or every service of a project.",
            Verb::Stop => "Stop a container, a service, or every service of a project.",
            Verb::Restart => "Stop and start a container, a service, or a project again.",
            Verb::Pause => "Freeze the processes of a container, a service, or a project.",
            Verb::Resume => "Resume the frozen processes of a container, a service, or a project.",
            Verb::Up => "Create and start the services of a project (docker compose up).",
            Verb::Down => "Stop and remove the containers of a project. Captain asks first.",
            Verb::Logs => "Show the logs of a container, or the log of a project.",
            Verb::Shell => "Open a shell in a container.",
            Verb::Open => "Open a published port in your browser.",
            Verb::Forward => "Forward a Kubernetes service to a port on this computer.",
            Verb::Float => "Open the log of a container in a small window that stays on top.",
            Verb::Disk => "Show what fills the engine's disk, and free up space.",
            Verb::Go => "Go to a page.",
        }
    }

    /// What each word after the verb names, in order.
    pub(super) fn slots(self) -> &'static [Kinds] {
        match self {
            Verb::Start | Verb::Stop | Verb::Restart | Verb::Pause | Verb::Resume | Verb::Logs => {
                &[RUNNABLE]
            }
            Verb::Up => &[UP],
            Verb::Down => &[PROJECT],
            Verb::Shell | Verb::Float => &[CONTAINER],
            Verb::Open => &[OPENER],
            Verb::Forward => &[KUBE, PORT],
            Verb::Go => &[PAGE],
            Verb::Disk => &[],
        }
    }

    /// How many of the slots must be filled.
    pub(super) fn required(self) -> usize {
        self.slots().len().min(1)
    }

    /// The options the verb takes.
    pub fn flags(self) -> &'static [Flag] {
        match self {
            Verb::Logs => &Flag::ALL,
            _ => &[],
        }
    }
}
