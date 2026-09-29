use captain_core::format::age_label;
use captain_core::model::{Container, ContainerState};
use gpui_kit::component::table::Column;
use gpui_kit::component::tag::Tag;
use gpui_kit::*;

/// The columns of the container table, in display order.
#[derive(Debug, Clone, Copy)]
pub enum ContainerColumn {
    Name,
    Image,
    State,
    Status,
    Ports,
    Created,
}

impl ContainerColumn {
    pub const ALL: [ContainerColumn; 6] = [
        ContainerColumn::Name,
        ContainerColumn::Image,
        ContainerColumn::State,
        ContainerColumn::Status,
        ContainerColumn::Ports,
        ContainerColumn::Created,
    ];

    pub fn column(self) -> Column {
        let (key, name, width) = match self {
            ContainerColumn::Name => ("name", "Name", 180.),
            ContainerColumn::Image => ("image", "Image", 200.),
            ContainerColumn::State => ("state", "State", 100.),
            ContainerColumn::Status => ("status", "Status", 170.),
            ContainerColumn::Ports => ("ports", "Ports", 200.),
            ContainerColumn::Created => ("created", "Created", 110.),
        };
        Column::new(key, name).width(px(width))
    }

    /// `now` is the current Unix time in seconds, for the Created column.
    pub fn render_cell(self, container: &Container, now: i64) -> AnyElement {
        match self {
            ContainerColumn::Name => div().child(container.name.clone()).into_any_element(),
            ContainerColumn::Image => div().child(container.image.clone()).into_any_element(),
            ContainerColumn::State => state_tag(container.state).into_any_element(),
            ContainerColumn::Status => div().child(container.status.clone()).into_any_element(),
            ContainerColumn::Ports => div().child(container.ports_label()).into_any_element(),
            ContainerColumn::Created => div()
                .child(age_label(container.created, now))
                .into_any_element(),
        }
    }
}

fn state_tag(state: ContainerState) -> impl IntoElement {
    let tag = match state {
        ContainerState::Running => Tag::success(),
        ContainerState::Paused | ContainerState::Restarting => Tag::warning(),
        ContainerState::Dead => Tag::danger(),
        _ => Tag::secondary(),
    };
    tag.outline().child(state.label())
}
