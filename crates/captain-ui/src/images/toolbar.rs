use captain_core::format::bytes_label;
use captain_core::model::Image;
use gpui_kit::assets::IconName;
use gpui_kit::component::Sizable;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::*;

use super::toolbar_button::toolbar_button;
use super::{ImagesState, build_dialog};
use crate::help::HelpExt;
use crate::theme::Palette;

/// The pull field and button, Build, then Remove for the selected image and Prune
/// dangling.
pub fn render(
    handle: &Entity<ImagesState>,
    state: &ImagesState,
    input: &Entity<InputState>,
    palette: &Palette,
) -> impl IntoElement {
    let pulling = state.is_pulling();
    let pull = {
        let handle = handle.clone();
        let input = input.clone();
        toolbar_button(
            "pull-image",
            if pulling { "Pulling..." } else { "Pull" },
            IconName::Download,
            palette.accent,
            !pulling,
            palette,
            move |window, cx| start_pull(&handle, &input, window, cx),
        )
        .help("Pull the image named in the field, for example nginx:latest.")
    };

    let can_build = state.builder().is_some();
    let build = {
        let handle = handle.clone();
        let help = if can_build {
            "Build an image from a Dockerfile."
        } else {
            "Install Docker Buildx to build images."
        };
        toolbar_button(
            "build-image",
            "Build",
            IconName::Hammer,
            palette.accent,
            can_build,
            palette,
            move |window, cx| build_dialog::open(handle.clone(), window, cx),
        )
        .help(help)
    };
    let selected = state.selected();
    let remove_help = match selected {
        None => "Select an image to remove it.".to_string(),
        Some(image) if image.in_use() => {
            format!(
                "{} is in use by a container, so it stays.",
                image_name(image)
            )
        }
        Some(image) => format!("Remove the image {}.", image_name(image)),
    };
    let removable = selected.is_some_and(|image| !image.in_use() && !state.is_removing(&image.id));
    let remove = {
        let handle = handle.clone();
        let id = selected.map(|image| image.id.clone());
        let generation = state.generation();
        toolbar_button(
            "remove-image",
            "Remove",
            IconName::Trash,
            palette.red,
            removable,
            palette,
            move |_, cx| {
                if let Some(id) = id.clone() {
                    handle.update(cx, |state, cx| state.remove(id, generation, cx));
                }
            },
        )
        .help(remove_help)
    };

    let reclaimable = state.store().dangling_size();
    let prune_label = match (state.is_pruning(), reclaimable) {
        (true, _) => "Pruning...".to_string(),
        (false, 0) => "Prune dangling".to_string(),
        (false, bytes) => format!("Prune dangling · {}", bytes_label(bytes)),
    };
    let prune = {
        let handle = handle.clone();
        toolbar_button(
            "prune-images",
            prune_label,
            IconName::Eraser,
            palette.text,
            reclaimable > 0 && !state.is_pruning(),
            palette,
            move |_, cx| handle.update(cx, |state, cx| state.prune_dangling(cx)),
        )
        .help("Remove the untagged images that no container uses.")
    };

    div()
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(8.))
        .px(px(24.))
        .pb(px(12.))
        .child(
            div()
                .w(px(280.))
                .flex_shrink_0()
                .child(Input::new(input).small().disabled(pulling)),
        )
        .child(pull)
        .child(build)
        .child(div().flex_1())
        .child(remove)
        .child(prune)
}

/// The image's first tag, else its short ID.
fn image_name(image: &Image) -> String {
    image
        .repo_tags
        .first()
        .cloned()
        .unwrap_or_else(|| image.short_id().to_string())
}

/// Starts a pull of what the field holds, then clears the field if the pull started.
pub fn start_pull(
    handle: &Entity<ImagesState>,
    input: &Entity<InputState>,
    window: &mut Window,
    cx: &mut App,
) {
    let value = input.read(cx).value();
    handle.update(cx, |state, cx| state.pull_image(&value, cx));
    if handle.read(cx).is_pulling() {
        input.update(cx, |input, cx| input.clean(window, cx));
    }
}
