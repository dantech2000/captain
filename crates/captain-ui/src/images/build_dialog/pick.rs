use std::path::{Path, PathBuf};

use gpui_kit::*;

use super::BuildDialog;

impl BuildDialog {
    /// Asks for the context folder with the system dialog.
    pub(super) fn choose_context(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input = self.context.clone();
        pick(false, window, cx, move |path, _, window, cx| {
            let value = path.display().to_string();
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        });
    }

    /// Asks for the Dockerfile. A file in the context folder is shown relative to it.
    pub(super) fn choose_dockerfile(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input = self.dockerfile.clone();
        pick(true, window, cx, move |path, this, window, cx| {
            let context = PathBuf::from(this.context.read(cx).value().trim());
            let shown = relative_to(&path, &context).display().to_string();
            input.update(cx, |input, cx| input.set_value(shown, window, cx));
        });
    }
}

/// Opens the system dialog for one file (`files`) or one folder, and hands the chosen
/// path to `chosen`. Cancel does nothing.
fn pick(
    files: bool,
    window: &mut Window,
    cx: &mut Context<BuildDialog>,
    chosen: impl FnOnce(PathBuf, &mut BuildDialog, &mut Window, &mut Context<BuildDialog>) + 'static,
) {
    let paths = cx.prompt_for_paths(PathPromptOptions {
        files,
        directories: !files,
        multiple: false,
        prompt: Some("Choose".into()),
    });
    cx.spawn_in(window, async move |this, cx| {
        let Ok(Ok(Some(paths))) = paths.await else {
            return;
        };
        let Some(path) = paths.into_iter().next() else {
            return;
        };
        this.update_in(cx, |this, window, cx| chosen(path, this, window, cx))
            .ok();
    })
    .detach();
}

/// `path` relative to `dir` when it is inside it, else `path` as it is.
fn relative_to(path: &Path, dir: &Path) -> PathBuf {
    match path.strip_prefix(dir) {
        Ok(relative) if !dir.as_os_str().is_empty() => relative.to_path_buf(),
        _ => path.to_path_buf(),
    }
}
