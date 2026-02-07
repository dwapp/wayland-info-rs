use serde::Serialize;
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols::xdg::xdg_output::zv1::client::{
    zxdg_output_manager_v1::{self, ZxdgOutputManagerV1},
    zxdg_output_v1::{self, ZxdgOutputV1},
};

use crate::app::{AppData, UserData};

// XDG output manager info structure
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XdgOutputManagerInfo {
    #[serde(skip_serializing)]
    pub(crate) name: u32,
    pub(crate) outputs: Vec<XdgOutputInfo>,
}

// XDG output info structure
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XdgOutputInfo {
    #[serde(skip_serializing)]
    pub(crate) output_id: u32,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) logical_x: i32,
    pub(crate) logical_y: i32,
    pub(crate) logical_width: i32,
    pub(crate) logical_height: i32,
}

impl AppData {
    pub(crate) fn add_xdg_output_manager(&mut self, name: u32) {
        self.xdg_output_managers.push(XdgOutputManagerInfo {
            name,
            outputs: Vec::new(),
        });
    }

    pub(crate) fn add_xdg_output(&mut self, manager_index: usize, output_id: u32) {
        if let Some(manager) = self.xdg_output_managers.get_mut(manager_index) {
            manager.outputs.push(XdgOutputInfo {
                output_id,
                name: String::new(),
                description: String::new(),
                logical_x: 0,
                logical_y: 0,
                logical_width: 0,
                logical_height: 0,
            });
        }
    }

    pub(crate) fn update_xdg_output_name(
        &mut self,
        manager_index: usize,
        output_index: usize,
        name: String,
    ) {
        if let Some(manager) = self.xdg_output_managers.get_mut(manager_index) {
            if let Some(output) = manager.outputs.get_mut(output_index) {
                output.name = name;
            }
        }
    }

    pub(crate) fn update_xdg_output_description(
        &mut self,
        manager_index: usize,
        output_index: usize,
        description: String,
    ) {
        if let Some(manager) = self.xdg_output_managers.get_mut(manager_index) {
            if let Some(output) = manager.outputs.get_mut(output_index) {
                output.description = description;
            }
        }
    }

    pub(crate) fn update_xdg_output_logical_position(
        &mut self,
        manager_index: usize,
        output_index: usize,
        x: i32,
        y: i32,
    ) {
        if let Some(manager) = self.xdg_output_managers.get_mut(manager_index) {
            if let Some(output) = manager.outputs.get_mut(output_index) {
                output.logical_x = x;
                output.logical_y = y;
            }
        }
    }

    pub(crate) fn update_xdg_output_logical_size(
        &mut self,
        manager_index: usize,
        output_index: usize,
        width: i32,
        height: i32,
    ) {
        if let Some(manager) = self.xdg_output_managers.get_mut(manager_index) {
            if let Some(output) = manager.outputs.get_mut(output_index) {
                output.logical_width = width;
                output.logical_height = height;
            }
        }
    }
}

// Handle zxdg_output_manager_v1 events
impl Dispatch<ZxdgOutputManagerV1, UserData> for AppData {
    fn event(
        _state: &mut Self,
        _manager: &ZxdgOutputManagerV1,
        _event: zxdg_output_manager_v1::Event,
        _data: &UserData,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
    ) {
    }
}

// Handle zxdg_output_v1 events
impl Dispatch<ZxdgOutputV1, UserData> for AppData {
    fn event(
        state: &mut Self,
        _output: &ZxdgOutputV1,
        event: zxdg_output_v1::Event,
        data: &UserData,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
    ) {
        if let UserData::XdgOutput {
            manager_index,
            output_index,
        } = data
        {
            match event {
                zxdg_output_v1::Event::Name { name } => {
                    state.update_xdg_output_name(*manager_index, *output_index, name);
                }
                zxdg_output_v1::Event::Description { description } => {
                    state.update_xdg_output_description(*manager_index, *output_index, description);
                }
                zxdg_output_v1::Event::LogicalPosition { x, y } => {
                    state.update_xdg_output_logical_position(*manager_index, *output_index, x, y);
                }
                zxdg_output_v1::Event::LogicalSize { width, height } => {
                    state.update_xdg_output_logical_size(
                        *manager_index,
                        *output_index,
                        width,
                        height,
                    );
                }
                zxdg_output_v1::Event::Done => {}
                _ => {}
            }
        }
    }
}
