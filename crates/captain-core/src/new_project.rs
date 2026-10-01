//! Making new Compose projects: the built-in templates, the files of a project
//! made from an image or a `docker run` command, Docker Hub search, and the
//! checks of the New sheet's forms. See docs/features/0040-new-projects.md.

mod compose_text;
mod docker_hub;
mod docker_run;
mod free_port;
mod hub_rank;
mod project_dir;
mod project_name;
mod secret;
mod templates;

pub use compose_text::{ComposeDoc, ServiceSpec, named_volume};
pub use docker_hub::{
    DockerHub, HubError, HubRepo, count_label, parse_search, parse_tags, search_url, status_error,
    tags_url,
};
pub use docker_run::{RunConversion, convert_docker_run};
pub use free_port::{host_port_free, suggest_port};
pub use hub_rank::rank_search;
pub use project_dir::{NewFile, NewProjectError, folder_is_free, write_project};
pub use project_name::{
    image_project_name, project_name_error, to_project_name, unique_project_name,
};
pub use secret::{password_error, random_password};
pub use templates::{TEMPLATES, Template, TemplatePort, TemplateValues};
