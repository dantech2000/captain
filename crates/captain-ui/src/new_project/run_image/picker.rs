use std::time::Duration;

use captain_core::new_project::{HubError, HubRepo};
use gpui_kit::component::IndexPath;
use gpui_kit::component::list::{ListDelegate, ListItem, ListState};
use gpui_kit::component::skeleton::Skeleton;
use gpui_kit::*;

use super::picker_rows::{hub_row, two_lines};
use crate::help::HelpExt;
use crate::new_project::hub;
use crate::theme::Palette;

/// Docker Hub is asked this long after the last key press.
const DEBOUNCE: Duration = Duration::from_millis(400);
/// At most this many of the engine's images match one search.
const LOCAL_LIMIT: usize = 30;

/// What the user picked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Choice {
    /// An image on this engine, as `repository:tag` or an ID.
    Local(String),
    Hub(HubRepo),
    /// The search text as typed, for an image the search did not find.
    Typed(String),
}

/// The Docker Hub section of the list.
enum HubRows {
    Idle,
    Loading,
    Ready(Vec<HubRepo>),
    /// Why there are no results, such as offline or a 429.
    Note(String),
}

/// The image list: the engine's images first, then Docker Hub, with search.
pub struct ImagePicker {
    local: Vec<String>,
    query: String,
    local_hits: Vec<String>,
    hub: HubRows,
}

impl ImagePicker {
    pub fn new() -> Self {
        Self {
            local: Vec::new(),
            query: String::new(),
            local_hits: Vec::new(),
            hub: HubRows::Idle,
        }
    }

    /// Sets the engine's image references, once they load.
    pub fn set_local(&mut self, local: Vec<String>) {
        self.local = local;
        self.match_local();
    }

    fn match_local(&mut self) {
        let query = self.query.to_lowercase();
        self.local_hits = self
            .local
            .iter()
            .filter(|reference| reference.to_lowercase().contains(&query))
            .take(LOCAL_LIMIT)
            .cloned()
            .collect();
    }

    /// True when the first row offers the search text itself.
    fn offers_typed(&self) -> bool {
        !self.query.is_empty() && !self.local_hits.contains(&self.query)
    }

    /// The choice at `ix`, or `None` for a placeholder row.
    pub fn choice(&self, ix: IndexPath) -> Option<Choice> {
        if ix.section == 1 {
            return match &self.hub {
                HubRows::Ready(repos) => repos.get(ix.row).cloned().map(Choice::Hub),
                _ => None,
            };
        }
        let typed = usize::from(self.offers_typed());
        if ix.row < typed {
            return Some(Choice::Typed(self.query.clone()));
        }
        self.local_hits
            .get(ix.row - typed)
            .cloned()
            .map(Choice::Local)
    }

    /// Asks Docker Hub for `query` after the pause, unless the answer is kept.
    fn search_hub(
        &mut self,
        query: String,
        window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Task<()> {
        if query.len() < 2 {
            self.hub = HubRows::Idle;
            return Task::ready(());
        }
        if let Some(repos) = hub::cached_search(&query, cx) {
            self.hub = HubRows::Ready(repos);
            return Task::ready(());
        }
        let client = match hub::client(cx) {
            Ok(client) => client,
            Err(note) => {
                self.hub = HubRows::Note(note);
                return Task::ready(());
            }
        };
        self.hub = HubRows::Loading;
        cx.spawn_in(window, async move |this, cx| {
            cx.background_executor().timer(DEBOUNCE).await;
            let result = client.search(&query).await;
            this.update(cx, |list, cx| {
                let rows = match result {
                    Ok(repos) => {
                        hub::keep_search(query, repos.clone(), cx);
                        HubRows::Ready(repos)
                    }
                    Err(HubError::RateLimited(wait)) => {
                        hub::wait(wait, cx);
                        HubRows::Note(HubError::RateLimited(wait).to_string())
                    }
                    Err(error) => HubRows::Note(error.to_string()),
                };
                list.delegate_mut().hub = rows;
                cx.notify();
            })
            .ok();
        })
    }
}

impl ListDelegate for ImagePicker {
    type Item = ListItem;

    fn perform_search(
        &mut self,
        query: &str,
        window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Task<()> {
        self.query = query.trim().to_string();
        self.match_local();
        self.search_hub(self.query.clone(), window, cx)
    }

    fn sections_count(&self, _: &App) -> usize {
        2
    }

    fn items_count(&self, section: usize, _: &App) -> usize {
        match section {
            0 => usize::from(self.offers_typed()) + self.local_hits.len(),
            _ => match &self.hub {
                HubRows::Idle => 0,
                HubRows::Loading => 3,
                HubRows::Ready(repos) => repos.len(),
                HubRows::Note(_) => 1,
            },
        }
    }

    fn render_section_header(
        &mut self,
        section: usize,
        _: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<impl IntoElement> {
        let palette = Palette::of(cx);
        Some(
            div()
                .h(px(24.))
                .px(px(8.))
                .flex()
                .items_end()
                .text_size(px(11.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.text3)
                .child(if section == 0 {
                    "On this engine"
                } else {
                    "Docker Hub"
                }),
        )
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<ListItem> {
        let palette = Palette::of(cx);
        let id = ("image-pick", ix.section * 1000 + ix.row);
        let item = ListItem::new(id);
        let row = |title: SharedString, line: SharedString, help: String| {
            two_lines(ix, title, line, &palette).help(help)
        };
        Some(match (ix.section, self.choice(ix)) {
            (_, Some(Choice::Typed(text))) => item.child(row(
                format!("Use \u{201c}{text}\u{201d}").into(),
                "An image name with an optional registry and tag.".into(),
                format!(
                    "Use {text} as the image. Captain pulls it if this engine does not have it."
                ),
            )),
            (_, Some(Choice::Local(reference))) => item.child(row(
                reference.clone().into(),
                "On this engine".into(),
                format!("Run {reference}, which this engine has."),
            )),
            (_, Some(Choice::Hub(repo))) => item.child(hub_row(ix, &repo, &palette)),
            _ => match &self.hub {
                HubRows::Note(note) => item.disabled(true).child(row(
                    "No Docker Hub results".into(),
                    note.clone().into(),
                    format!("{note} The engine's images still show."),
                )),
                _ => item.disabled(true).child(
                    div()
                        .h(px(36.))
                        .flex()
                        .flex_col()
                        .justify_center()
                        .gap(px(6.))
                        .child(Skeleton::new().h(px(12.)).w(px(160.)))
                        .child(Skeleton::new().secondary().h(px(10.)).w(px(260.))),
                ),
            },
        })
    }

    fn set_selected_index(
        &mut self,
        _: Option<IndexPath>,
        _: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) {
        cx.notify();
    }
}
