//! `ddClient.desktopUI.navigate`: the SDK's `NavigationIntents`, which open a page
//! of the app. See
//! <https://docs.docker.com/reference/api/extensions-sdk/NavigationIntents/>.

use serde_json::Value;

/// One `navigate.view*` call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigateIntent {
    Containers,
    Container {
        id: String,
        view: ContainerView,
    },
    Images,
    /// `viewImage(id, tag)`. The tag only names the image in the SDK's page.
    Image {
        id: String,
        tag: String,
    },
    Volumes,
    Volume(String),
}

/// The part of a container's page a call opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerView {
    /// `viewContainer` and `viewContainerInspect`: the details.
    Details,
    Logs,
    Terminal,
    Stats,
}

impl NavigateIntent {
    /// Parses `name`, the method after `desktopUI.navigate.`. The SDK's promise
    /// fails for an object that does not exist, so a call with an ID needs one.
    pub fn parse(name: &str, params: &Value) -> Result<Self, String> {
        let text = |key: &str| {
            params
                .get(key)
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .map(str::to_string)
                .ok_or_else(|| format!("navigate.{name} needs \"{key}\""))
        };
        let container = |view| {
            Ok(Self::Container {
                id: text("id")?,
                view,
            })
        };
        match name {
            "viewContainers" => Ok(Self::Containers),
            "viewContainer" | "viewContainerInspect" => container(ContainerView::Details),
            "viewContainerLogs" => container(ContainerView::Logs),
            "viewContainerTerminal" => container(ContainerView::Terminal),
            "viewContainerStats" => container(ContainerView::Stats),
            "viewImages" => Ok(Self::Images),
            "viewImage" => Ok(Self::Image {
                id: text("id")?,
                tag: params
                    .get("tag")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            }),
            "viewVolumes" => Ok(Self::Volumes),
            "viewVolume" => text("volume").map(Self::Volume),
            _ => Err(format!(
                "desktopUI.navigate.{name} is not supported by Captain"
            )),
        }
    }

    /// Whether the engine must find the container, image, or volume first.
    pub fn names_object(&self) -> bool {
        matches!(
            self,
            Self::Container { .. } | Self::Image { .. } | Self::Volume(_)
        )
    }
}

#[cfg(test)]
mod tests;
