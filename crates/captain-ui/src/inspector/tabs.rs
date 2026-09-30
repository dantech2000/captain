use gpui_kit::*;

use crate::help::HelpExt;
use crate::theme::Palette;

/// The inspector tabs, in display order.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Overview,
    Logs,
    Terminal,
    Files,
    Stats,
}

impl Tab {
    pub const ALL: [Tab; 5] = [
        Tab::Overview,
        Tab::Logs,
        Tab::Terminal,
        Tab::Files,
        Tab::Stats,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Tab::Overview => "Overview",
            Tab::Logs => "Logs",
            Tab::Terminal => "Terminal",
            Tab::Files => "Files",
            Tab::Stats => "Stats",
        }
    }

    fn help(self) -> &'static str {
        match self {
            Tab::Overview => "Show the state, ports, mounts, and settings of the container.",
            Tab::Logs => "Show the output of the container, with search and level filters.",
            Tab::Terminal => "Open a shell in the container.",
            Tab::Files => "Browse the files in the container and save them to this computer.",
            Tab::Stats => "Show the CPU, memory, and network use of the container over time.",
        }
    }
}

/// An underlined tab bar. `on_select` receives the chosen tab.
pub fn render(
    current: Tab,
    palette: &Palette,
    on_select: impl Fn(Tab, &mut Window, &mut App) + Clone + 'static,
) -> impl IntoElement {
    div()
        .flex()
        .gap(px(18.))
        .border_b_1()
        .border_color(palette.sep)
        .children(Tab::ALL.into_iter().map(|tab| {
            let on_select = on_select.clone();
            let selected = tab == current;
            div()
                .id(tab.label())
                .pb(px(9.))
                .border_b_2()
                .border_color(if selected {
                    palette.accent
                } else {
                    transparent_black()
                })
                .text_size(px(12.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(if selected {
                    palette.text
                } else {
                    palette.text2
                })
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_click(move |_, window, cx| on_select(tab, window, cx))
                .child(tab.label())
                .help(tab.help())
        }))
}
