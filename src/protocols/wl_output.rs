use serde::Serialize;
use wayland_client::{
    protocol::{wl_output, wl_output::WlOutput},
    Connection, Dispatch, QueueHandle, WEnum,
};

use crate::app::{AppData, UserData};

// Output info structure
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputInfo {
    #[serde(skip_serializing)]
    pub(crate) name: u32,
    pub(crate) output_name: String,
    pub(crate) description: String,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) scale: i32,
    pub(crate) physical_width: i32,
    pub(crate) physical_height: i32,
    pub(crate) make: String,
    pub(crate) model: String,
    pub(crate) subpixel_orientation: String,
    pub(crate) output_transform: String,
    pub(crate) modes: Vec<OutputMode>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputMode {
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) refresh: i32,
    pub(crate) flags: Vec<String>,
}

// Output geometry info structure to reduce function parameters
#[derive(Debug)]
pub struct OutputGeometry {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) physical_width: i32,
    pub(crate) physical_height: i32,
    pub(crate) subpixel: WEnum<wl_output::Subpixel>,
    pub(crate) transform: WEnum<wl_output::Transform>,
    pub(crate) make: String,
    pub(crate) model: String,
}

impl AppData {
    pub(crate) fn add_output(&mut self, name: u32, output_name: String) {
        self.outputs.push(OutputInfo {
            name,
            output_name,
            description: String::new(),
            x: 0,
            y: 0,
            scale: 1,
            physical_width: 0,
            physical_height: 0,
            make: String::new(),
            model: String::new(),
            subpixel_orientation: String::new(),
            output_transform: String::new(),
            modes: Vec::new(),
        });
    }

    pub(crate) fn update_output_geometry(&mut self, output_index: usize, geometry: OutputGeometry) {
        if let Some(output) = self.outputs.get_mut(output_index) {
            output.x = geometry.x;
            output.y = geometry.y;
            output.physical_width = geometry.physical_width;
            output.physical_height = geometry.physical_height;
            output.make = geometry.make;
            output.model = geometry.model;

            output.subpixel_orientation = match geometry.subpixel {
                WEnum::Value(wl_output::Subpixel::Unknown) => "unknown".to_string(),
                WEnum::Value(wl_output::Subpixel::None) => "none".to_string(),
                WEnum::Value(wl_output::Subpixel::HorizontalRgb) => "horizontal_rgb".to_string(),
                WEnum::Value(wl_output::Subpixel::HorizontalBgr) => "horizontal_bgr".to_string(),
                WEnum::Value(wl_output::Subpixel::VerticalRgb) => "vertical_rgb".to_string(),
                WEnum::Value(wl_output::Subpixel::VerticalBgr) => "vertical_bgr".to_string(),
                _ => "unknown".to_string(),
            };

            output.output_transform = match geometry.transform {
                WEnum::Value(wl_output::Transform::Normal) => "normal".to_string(),
                WEnum::Value(wl_output::Transform::_90) => "90".to_string(),
                WEnum::Value(wl_output::Transform::_180) => "180".to_string(),
                WEnum::Value(wl_output::Transform::_270) => "270".to_string(),
                WEnum::Value(wl_output::Transform::Flipped) => "flipped".to_string(),
                WEnum::Value(wl_output::Transform::Flipped90) => "flipped-90".to_string(),
                WEnum::Value(wl_output::Transform::Flipped180) => "flipped-180".to_string(),
                WEnum::Value(wl_output::Transform::Flipped270) => "flipped-270".to_string(),
                _ => "normal".to_string(),
            };
        }
    }

    pub(crate) fn update_output_mode(
        &mut self,
        output_index: usize,
        flags: WEnum<wl_output::Mode>,
        width: i32,
        height: i32,
        refresh: i32,
    ) {
        if let Some(output) = self.outputs.get_mut(output_index) {
            let mut mode_flags = Vec::new();
            match flags {
                WEnum::Value(wl_output::Mode::Current) => mode_flags.push("current".to_string()),
                WEnum::Value(wl_output::Mode::Preferred) => {
                    mode_flags.push("preferred".to_string())
                }
                _ => {}
            }

            output.modes.push(OutputMode {
                width,
                height,
                refresh,
                flags: mode_flags,
            });
        }
    }

    pub(crate) fn update_output_scale(&mut self, output_index: usize, factor: i32) {
        if let Some(output) = self.outputs.get_mut(output_index) {
            output.scale = factor;
        }
    }

    pub(crate) fn update_output_name(&mut self, output_index: usize, name: String) {
        if let Some(output) = self.outputs.get_mut(output_index) {
            output.output_name = name;
        }
    }

    pub(crate) fn update_output_description(&mut self, output_index: usize, description: String) {
        if let Some(output) = self.outputs.get_mut(output_index) {
            output.description = description;
        }
    }
}

// Handle wl_output events
impl Dispatch<WlOutput, UserData> for AppData {
    fn event(
        state: &mut Self,
        _output: &WlOutput,
        event: wl_output::Event,
        data: &UserData,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
    ) {
        if let UserData::Output { output_index } = data {
            match event {
                wl_output::Event::Geometry {
                    x,
                    y,
                    physical_width,
                    physical_height,
                    subpixel,
                    make,
                    model,
                    transform,
                } => {
                    let geometry = OutputGeometry {
                        x,
                        y,
                        physical_width,
                        physical_height,
                        subpixel,
                        transform,
                        make,
                        model,
                    };
                    state.update_output_geometry(*output_index, geometry);
                }
                wl_output::Event::Mode {
                    flags,
                    width,
                    height,
                    refresh,
                } => {
                    state.update_output_mode(*output_index, flags, width, height, refresh);
                }
                wl_output::Event::Scale { factor } => {
                    state.update_output_scale(*output_index, factor);
                }
                wl_output::Event::Name { name } => {
                    state.update_output_name(*output_index, name);
                }
                wl_output::Event::Description { description } => {
                    state.update_output_description(*output_index, description);
                }
                _ => {}
            }
        }
    }
}
