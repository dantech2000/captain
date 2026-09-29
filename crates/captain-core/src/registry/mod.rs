//! Registry logins for push: the Docker config file, credential helper output, and
//! the hint for a refused push. Reading files and running helpers is up to the
//! caller.

mod auth;
mod docker_config;
mod host;
mod login_hint;

pub use auth::RegistryAuth;
pub use docker_config::{CredentialSource, DockerConfig};
pub use host::{DOCKER_HUB, registry_host, server_address};
pub use login_hint::push_error;
