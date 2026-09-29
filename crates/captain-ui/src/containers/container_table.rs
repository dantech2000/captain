use std::time::{SystemTime, UNIX_EPOCH};

use captain_core::model::Container;
use captain_core::store::ContainerStore;
use gpui_kit::component::table::{Column, TableDelegate, TableState};
use gpui_kit::*;

use super::columns::ContainerColumn;

/// Feeds the container store into the GPUI Kit data table.
#[derive(Default)]
pub struct ContainerTable {
    store: ContainerStore,
}

impl ContainerTable {
    pub fn store(&self) -> &ContainerStore {
        &self.store
    }

    pub fn replace(&mut self, containers: Vec<Container>) {
        self.store.replace(containers);
    }
}

impl TableDelegate for ContainerTable {
    fn columns_count(&self, _: &App) -> usize {
        ContainerColumn::ALL.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.store.len()
    }

    fn column(&self, col_ix: usize, _: &App) -> Column {
        ContainerColumn::ALL[col_ix].column()
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let Some(container) = self.store.get(row_ix) else {
            return div().into_any_element();
        };
        ContainerColumn::ALL[col_ix].render_cell(container, unix_now())
    }
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or_default()
}
