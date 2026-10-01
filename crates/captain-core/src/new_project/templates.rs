//! The built-in templates: Compose files with pinned tags, read from the
//! official-images library files on 2026-10-01
//! (<https://github.com/docker-library/official-images/tree/master/library>).
//! Passwords go in `.env`, never in `compose.yaml`.

mod texts;

use super::NewFile;

/// One port a template publishes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemplatePort {
    pub container: u16,
    /// What the port is for, for the form's label.
    pub label: &'static str,
}

/// A built-in template.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Template {
    pub title: &'static str,
    /// The line under the title in the picker.
    pub sentence: &'static str,
    pub image: &'static str,
    /// The service name, and the project name the form starts with.
    pub service: &'static str,
    pub ports: &'static [TemplatePort],
    /// The variable for the user name and its default, when the image takes one.
    pub user: Option<(&'static str, &'static str)>,
    /// The variable for the password, when the image needs one.
    pub password: Option<&'static str>,
    /// The `compose.yaml` text with `{name}`, `{port0}`, and `{port1}` in it.
    text: &'static str,
}

/// What the form gives a template.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TemplateValues {
    pub name: String,
    /// The host ports, one for each of [`Template::ports`].
    pub ports: Vec<u16>,
    pub user: String,
    pub password: String,
}

const fn port(container: u16, label: &'static str) -> TemplatePort {
    TemplatePort { container, label }
}

/// The templates, in the picker's order.
pub const TEMPLATES: [Template; 6] = [
    Template {
        title: "PostgreSQL",
        sentence: "PostgreSQL 18 with its data in a volume, and psql tasks.",
        image: "postgres:18",
        service: "postgres",
        ports: &[port(5432, "Port")],
        user: None,
        password: Some("POSTGRES_PASSWORD"),
        text: texts::POSTGRES,
    },
    Template {
        title: "MySQL",
        sentence: "MySQL 8.4 LTS with its data in a volume.",
        image: "mysql:8.4",
        service: "mysql",
        ports: &[port(3306, "Port")],
        user: None,
        password: Some("MYSQL_ROOT_PASSWORD"),
        text: texts::MYSQL,
    },
    Template {
        title: "Redis",
        sentence: "Redis 8 on Alpine, saving to a volume with an append-only file.",
        image: "redis:8-alpine",
        service: "redis",
        ports: &[port(6379, "Port")],
        user: None,
        password: None,
        text: texts::REDIS,
    },
    Template {
        title: "MongoDB",
        sentence: "MongoDB 8 with a root user and its data in a volume.",
        image: "mongo:8",
        service: "mongo",
        ports: &[port(27017, "Port")],
        user: Some(("MONGO_INITDB_ROOT_USERNAME", "admin")),
        password: Some("MONGO_INITDB_ROOT_PASSWORD"),
        text: texts::MONGO,
    },
    Template {
        title: "RabbitMQ",
        sentence: "RabbitMQ 4 with the management web page.",
        image: "rabbitmq:4-management",
        service: "rabbitmq",
        ports: &[port(5672, "AMQP port"), port(15672, "Web page port")],
        user: Some(("RABBITMQ_DEFAULT_USER", "admin")),
        password: Some("RABBITMQ_DEFAULT_PASS"),
        text: texts::RABBITMQ,
    },
    Template {
        title: "Web server",
        sentence: "nginx serving the site folder of the project, read-only.",
        image: "nginx:stable-alpine",
        service: "web",
        ports: &[port(80, "Port")],
        user: None,
        password: None,
        text: texts::NGINX,
    },
];

impl Template {
    /// The host port the form suggests first for port `index`: the container
    /// port, or 8080 for a web server's port 80.
    pub fn preferred_port(&self, index: usize) -> u16 {
        match self.ports[index].container {
            80 => 8080,
            other => other,
        }
    }

    /// The project's files: `compose.yaml`, then `.env` (mode 0600) and a
    /// `.gitignore` that keeps it out of Git when there is a password, and the
    /// web server's starter page.
    pub fn files(&self, values: &TemplateValues) -> Vec<NewFile> {
        let mut compose = self.text.replace("{name}", &values.name);
        for (index, port) in values.ports.iter().enumerate() {
            compose = compose.replace(&format!("{{port{index}}}"), &port.to_string());
        }
        let mut files = vec![NewFile::new("compose.yaml", compose)];
        if let Some(password) = self.password {
            let mut env = String::from(
                "# Passwords for this project. Compose reads this file. Keep it out of version control.\n",
            );
            if let Some((user, _)) = self.user {
                env.push_str(&format!("{user}={}\n", values.user));
            }
            env.push_str(&format!("{password}={}\n", values.password));
            files.push(NewFile::private(".env", env));
            files.push(NewFile::new(".gitignore", ".env\n"));
        }
        if self.image.starts_with("nginx:") {
            files.push(NewFile::new("site/index.html", texts::INDEX_HTML));
        }
        files
    }
}

#[cfg(test)]
mod tests;
