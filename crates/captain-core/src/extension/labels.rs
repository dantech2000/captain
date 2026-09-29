//! The image labels that describe an extension. See
//! <https://docs.docker.com/extensions/extensions-sdk/extensions/labels/>.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Every extension image has this label. Captain refuses images without it.
pub const API_VERSION_LABEL: &str = "com.docker.desktop.extension.api.version";
const TITLE_LABEL: &str = "org.opencontainers.image.title";
const DESCRIPTION_LABEL: &str = "org.opencontainers.image.description";
const VENDOR_LABEL: &str = "org.opencontainers.image.vendor";
const PUBLISHER_URL_LABEL: &str = "com.docker.extension.publisher-url";

/// The labels Captain shows.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionLabels {
    pub api_version: String,
    pub title: String,
    pub description: String,
    /// The publisher: a person or an organization.
    pub vendor: String,
    pub publisher_url: String,
}

impl ExtensionLabels {
    /// Reads the labels of an image. Fails when the image is not an extension.
    pub fn from_image(labels: &HashMap<String, String>) -> Result<Self, String> {
        let get = |key: &str| labels.get(key).map(|v| v.trim().to_string());
        let api_version = get(API_VERSION_LABEL)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| {
                format!("The image is not a Docker extension: it has no {API_VERSION_LABEL} label.")
            })?;
        Ok(Self {
            api_version,
            title: get(TITLE_LABEL).unwrap_or_default(),
            description: get(DESCRIPTION_LABEL).unwrap_or_default(),
            vendor: get(VENDOR_LABEL).unwrap_or_default(),
            publisher_url: get(PUBLISHER_URL_LABEL).unwrap_or_default(),
        })
    }

    /// The publisher for the install dialog, or a warning that the image names none.
    pub fn publisher(&self) -> &str {
        if self.vendor.is_empty() {
            "Unknown publisher"
        } else {
            &self.vendor
        }
    }
}

#[cfg(test)]
mod tests;
