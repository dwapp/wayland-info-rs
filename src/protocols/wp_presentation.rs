use serde::Serialize;
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols::wp::presentation_time::client::wp_presentation::{self, WpPresentation};

use crate::app::{AppData, UserData};

// Presentation info structure
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationInfo {
    #[serde(skip_serializing)]
    pub(crate) name: u32,
    pub(crate) clock_id: Option<u32>,
}

impl AppData {
    pub(crate) fn add_presentation(&mut self, name: u32) {
        self.presentation_info.push(PresentationInfo {
            name,
            clock_id: None,
        });
    }

    pub(crate) fn update_presentation_clock_id(
        &mut self,
        presentation_index: usize,
        clock_id: u32,
    ) {
        if let Some(presentation) = self.presentation_info.get_mut(presentation_index) {
            presentation.clock_id = Some(clock_id);
        }
    }
}

// Handle wp_presentation events
impl Dispatch<WpPresentation, UserData> for AppData {
    fn event(
        state: &mut Self,
        _presentation: &WpPresentation,
        event: wp_presentation::Event,
        data: &UserData,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
    ) {
        if let UserData::Presentation { presentation_index } = data {
            if let wp_presentation::Event::ClockId { clk_id } = event {
                state.update_presentation_clock_id(*presentation_index, clk_id);
            }
        }
    }
}
