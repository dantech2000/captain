use std::sync::Once;

use gpui_kit::component::highlighter::{LanguageConfig, LanguageRegistry};

/// The editor language name of Dockerfiles.
pub const DOCKERFILE: &str = "dockerfile";

/// Registers the Dockerfile grammar, `tree-sitter-containerfile`, with the
/// highlighter. GPUI Kit bundles no Dockerfile grammar. Safe to call often.
pub fn register_dockerfile() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let config = LanguageConfig::new(
            DOCKERFILE,
            tree_sitter_containerfile::LANGUAGE.into(),
            Vec::new(),
            tree_sitter_containerfile::HIGHLIGHTS_QUERY,
            "",
            "",
        );
        LanguageRegistry::singleton().register(DOCKERFILE, &config);
    });
}

#[cfg(test)]
mod tests;
