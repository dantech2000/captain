//! Completion and hover docs for Compose files, from the vendored Compose schema
//! and Captain's `x-captain` keys.

use anyhow::Result;
use captain_core::project_files::{ComposeSchema, cursor_context};
use gpui_kit::component::input::{CompletionProvider, HoverProvider, Rope, RopeExt};
use gpui_kit::*;
use lsp_types::{
    CompletionContext, CompletionItem, CompletionItemKind, CompletionResponse, CompletionTextEdit,
    Documentation, Hover, HoverContents, MarkupContent, MarkupKind, TextEdit,
};

/// Offers the keys the schema allows where the cursor is, and shows a key's
/// description on hover.
pub struct ComposeHelp;

impl CompletionProvider for ComposeHelp {
    fn completions(
        &self,
        text: &Rope,
        offset: usize,
        _: CompletionContext,
        _: &mut Window,
        _: &mut App,
    ) -> Task<Result<CompletionResponse>> {
        let context = cursor_context(&text.to_string(), offset);
        let Some(prefix) = context.prefix.filter(|prefix| !prefix.is_empty()) else {
            return Task::ready(Ok(CompletionResponse::Array(Vec::new())));
        };
        let range = lsp_types::Range {
            start: text.offset_to_position(offset - prefix.len()),
            end: text.offset_to_position(offset),
        };
        let items = ComposeSchema::bundled()
            .keys(&context.parents)
            .into_iter()
            .filter(|key| key.name.starts_with(&prefix) && key.name != prefix)
            .map(|key| CompletionItem {
                label: key.name.clone(),
                kind: Some(CompletionItemKind::PROPERTY),
                documentation: key.doc.map(Documentation::String),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range,
                    new_text: format!("{}: ", key.name),
                })),
                ..Default::default()
            })
            .collect();
        Task::ready(Ok(CompletionResponse::Array(items)))
    }

    fn is_completion_trigger(&self, _: usize, new_text: &str, _: &mut App) -> bool {
        new_text.len() == 1
            && new_text
                .chars()
                .all(|c| c.is_ascii_alphabetic() || c == '_')
    }
}

impl HoverProvider for ComposeHelp {
    fn hover(
        &self,
        text: &Rope,
        offset: usize,
        _: &mut Window,
        _: &mut App,
    ) -> Task<Result<Option<Hover>>> {
        let context = cursor_context(&text.to_string(), offset);
        let hover = context.hovered.and_then(|path| {
            let doc = ComposeSchema::bundled().doc(&path)?;
            let key = path.last()?;
            Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**{key}**\n\n{doc}"),
                }),
                range: None,
            })
        });
        Task::ready(Ok(hover))
    }
}
