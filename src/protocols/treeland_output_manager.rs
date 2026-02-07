use serde::Serialize;
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols_treeland::output_manager::v1::client::treeland_output_manager_v1::{
    self, TreelandOutputManagerV1,
};

use crate::app::{AppData, UserData};

// Treeland output manager info structure
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TreelandOutputManagerInfo {
    #[serde(skip_serializing)]
    pub(crate) name: u32,
    pub(crate) primary_output: Option<String>,
}

impl AppData {
    pub(crate) fn add_treeland_output_manager(&mut self, name: u32) {
        self.treeland_output_managers
            .push(TreelandOutputManagerInfo {
                name,
                primary_output: None,
            });
    }

    pub(crate) fn update_treeland_primary_output(
        &mut self,
        manager_index: usize,
        output_name: String,
    ) {
        if let Some(manager) = self.treeland_output_managers.get_mut(manager_index) {
            manager.primary_output = Some(output_name);
        }
    }
}

// Handle treeland_output_manager_v1 events
impl Dispatch<TreelandOutputManagerV1, UserData> for AppData {
    fn event(
        state: &mut Self,
        _manager: &TreelandOutputManagerV1,
        event: treeland_output_manager_v1::Event,
        data: &UserData,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
    ) {
        if let UserData::TreelandOutputManager { manager_index } = data {
            if let treeland_output_manager_v1::Event::PrimaryOutput { output_name } = event {
                state.update_treeland_primary_output(*manager_index, output_name);
            }
        }
    }
}
