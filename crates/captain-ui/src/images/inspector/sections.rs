use captain_core::format::bytes_label;
use captain_core::model::{Image, ImageDetail};
use gpui_kit::*;

use crate::theme::Palette;
use crate::widgets::{KeyValue, colored_key_values, section_note, titled_section};

/// ID, tags, digests, creation time, platform, and size.
pub fn details(image: &Image, detail: &ImageDetail, palette: &Palette) -> Div {
    let text = palette.text;
    let mut rows: Vec<KeyValue> = vec![("Image ID".into(), image.short_id().into(), text)];
    let tags = if detail.repo_tags.is_empty() {
        "none".to_string()
    } else {
        detail.repo_tags.join(", ")
    };
    rows.push(("Tags".into(), tags, text));
    rows.extend(detail.repo_digests.iter().map(|digest| {
        let short = digest.split_once('@').map_or(digest.as_str(), |(_, d)| d);
        ("Digest".to_string(), short.to_string(), palette.text2)
    }));
    rows.push(("Created".into(), detail.created_label(), text));
    rows.push(("Platform".into(), or_dash(detail.platform()), text));
    // The list size is the unpacked size on disk. With the containerd image store,
    // inspect reports the compressed content size, so show both and say which is which.
    rows.push(("Size".into(), bytes_label(image.size), text));
    if detail.size != image.size {
        rows.push((
            "Download size".into(),
            bytes_label(detail.size),
            palette.text2,
        ));
    }
    titled_section("Details", None, colored_key_values(rows, palette), palette)
}

/// Entrypoint, command, working directory, and user.
pub fn config(detail: &ImageDetail, palette: &Palette) -> Div {
    let config = &detail.config;
    let join = |parts: &[String]| or_dash(parts.join(" "));
    let rows = vec![
        ("Entrypoint".into(), join(&config.entrypoint), palette.text),
        ("Command".into(), join(&config.cmd), palette.text),
        (
            "Working dir".into(),
            or(&config.working_dir, "/"),
            palette.text,
        ),
        ("User".into(), or(&config.user, "root"), palette.text),
    ];
    titled_section("Config", None, colored_key_values(rows, palette), palette)
}

/// The ports the image exposes.
pub fn ports(detail: &ImageDetail, palette: &Palette) -> Div {
    let ports = &detail.config.exposed_ports;
    let body = if ports.is_empty() {
        section_note("No exposed ports", palette)
    } else {
        div()
            .flex()
            .flex_wrap()
            .gap(px(6.))
            .children(ports.iter().map(|port| {
                div()
                    .px(px(8.))
                    .py(px(3.))
                    .rounded(px(6.))
                    .bg(palette.field)
                    .font_family(palette.mono())
                    .text_size(px(11.))
                    .child(port.to_string())
            }))
    };
    titled_section("Exposed ports", None, body, palette)
}

/// The image's environment. Secret-looking values are masked.
pub fn environment(detail: &ImageDetail, palette: &Palette) -> Div {
    let rows: Vec<KeyValue> = detail
        .config
        .env
        .iter()
        .map(|var| {
            let color = if var.is_secret() {
                palette.text3
            } else {
                palette.text
            };
            (var.key.clone(), var.display_value(), color)
        })
        .collect();
    let body = if rows.is_empty() {
        section_note("No variables", palette)
    } else {
        colored_key_values(rows, palette)
    };
    titled_section("Environment", None, body, palette)
}

/// The image's labels, sorted by key.
pub fn labels(detail: &ImageDetail, palette: &Palette) -> Div {
    let rows: Vec<KeyValue> = detail
        .config
        .labels
        .iter()
        .map(|(key, value)| (key.clone(), value.clone(), palette.text2))
        .collect();
    let body = if rows.is_empty() {
        section_note("No labels", palette)
    } else {
        colored_key_values(rows, palette)
    };
    titled_section("Labels", None, body, palette)
}

fn or(value: &str, default: &str) -> String {
    if value.is_empty() {
        default.to_string()
    } else {
        value.to_string()
    }
}

fn or_dash(value: String) -> String {
    if value.is_empty() {
        "—".to_string()
    } else {
        value
    }
}
