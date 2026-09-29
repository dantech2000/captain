//! The app's side of the UI's [`SystemIntegration`]: the login item, the menu bar
//! icon, and administrative access. See feature 0015.

use std::path::Path;

use captain_core::behavior::docker_socket::SocketProbe;
use captain_ui::SystemIntegration;

#[cfg(unix)]
use crate::docker_socket;
use crate::login_item;

pub struct System;

impl SystemIntegration for System {
    fn login_item(&self) -> Result<bool, String> {
        login_item::is_enabled().map_err(|error| error.to_string())
    }

    fn set_login_item(&self, enabled: bool) -> Result<(), String> {
        tracing::info!(enabled, "changing the login item");
        login_item::set(enabled).map_err(|error| error.to_string())
    }

    fn has_menu_bar_icon(&self) -> bool {
        cfg!(any(target_os = "macos", target_os = "windows"))
    }

    #[cfg(unix)]
    fn docker_socket(&self) -> Option<SocketProbe> {
        Some(docker_socket::probe())
    }

    #[cfg(unix)]
    fn link_docker_socket(&self, target: &Path) -> Result<(), String> {
        docker_socket::link(target)
    }

    #[cfg(unix)]
    fn unlink_docker_socket(&self) -> Result<(), String> {
        docker_socket::unlink()
    }

    #[cfg(not(unix))]
    fn docker_socket(&self) -> Option<SocketProbe> {
        None
    }

    #[cfg(not(unix))]
    fn link_docker_socket(&self, _: &Path) -> Result<(), String> {
        Err("Windows has no /var/run/docker.sock.".into())
    }

    #[cfg(not(unix))]
    fn unlink_docker_socket(&self) -> Result<(), String> {
        Err("Windows has no /var/run/docker.sock.".into())
    }
}
