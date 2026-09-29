//! App behavior: the login item text and the `/var/run/docker.sock` link decisions.
//! The app does the file, registry, and privileged work. See feature 0015.

pub mod docker_socket;
pub mod login_item;
