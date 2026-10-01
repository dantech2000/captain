use captain_core::new_project::{TEMPLATES, Template};
use gpui_kit::assets::IconName;
use gpui_kit::component::IndexPath;
use gpui_kit::component::list::{ListDelegate, ListItem, ListState};
use gpui_kit::*;

use crate::help::HelpExt;
use crate::icons::glyph;
use crate::new_project::pick_row::{item, lines};
use crate::theme::Palette;

/// The template list with search: up and down move, Return picks.
pub struct TemplatePicker {
    /// Indexes into [`TEMPLATES`] that match the search.
    matches: Vec<usize>,
}

impl TemplatePicker {
    pub fn new() -> Self {
        Self {
            matches: (0..TEMPLATES.len()).collect(),
        }
    }

    /// The template at `ix` in the list.
    pub fn template(&self, ix: IndexPath) -> Option<&'static Template> {
        self.matches.get(ix.row).map(|index| &TEMPLATES[*index])
    }
}

/// The icon of a template's row.
fn icon(template: &Template) -> IconName {
    match template.service {
        "rabbitmq" => IconName::Rabbit,
        "web" => IconName::Globe,
        "redis" => IconName::Layers,
        _ => IconName::Database,
    }
}

impl ListDelegate for TemplatePicker {
    type Item = ListItem;

    fn perform_search(
        &mut self,
        query: &str,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) -> Task<()> {
        let query = query.to_lowercase();
        self.matches = TEMPLATES
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                [t.title, t.image, t.sentence]
                    .iter()
                    .any(|text| text.to_lowercase().contains(&query))
            })
            .map(|(index, _)| index)
            .collect();
        Task::ready(())
    }

    fn items_count(&self, _: usize, _: &App) -> usize {
        self.matches.len()
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<ListItem> {
        let template = self.template(ix)?;
        let palette = Palette::of(cx);
        let title = div().flex().gap(px(8.)).child(template.title).child(
            div()
                .text_color(palette.text3)
                .font_weight(FontWeight::NORMAL)
                .child(template.image),
        );
        let row = div()
            .id(("template-row", ix.row))
            .flex_1()
            .min_w_0()
            .flex()
            .items_center()
            .gap(px(10.))
            .child(glyph(icon(template), px(18.), palette.accent_fg))
            .child(lines(title, template.sentence, &palette))
            .help(format!(
                "Make a {} project from {}. Return or a click picks it.",
                template.title, template.image
            ));
        Some(item(("template", ix.row)).child(row))
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
