use serde::Serialize;
use wayland_client::{
    protocol::{wl_shm, wl_shm::WlShm},
    Connection, Dispatch, QueueHandle, WEnum,
};

use crate::app::{AppData, UserData};

// SHM info structure
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShmInfo {
    #[serde(skip_serializing)]
    pub(crate) name: u32,
    pub(crate) formats: Vec<ShmFormat>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShmFormat {
    pub(crate) format: u32,
    pub(crate) fourcc: String,
}

impl AppData {
    pub(crate) fn add_shm(&mut self, name: u32) {
        self.shm_info.push(ShmInfo {
            name,
            formats: Vec::new(),
        });
    }

    pub(crate) fn add_shm_format(&mut self, shm_index: usize, format: WEnum<wl_shm::Format>) {
        if let Some(shm) = self.shm_info.get_mut(shm_index) {
            let format_value = match format {
                WEnum::Value(value) => value as u32,
                WEnum::Unknown(value) => value,
            };
            let fourcc = format_to_fourcc(format_value);
            shm.formats.push(ShmFormat {
                format: format_value,
                fourcc,
            });
        }
    }
}

// Convert format code to FOURCC string
fn format_to_fourcc(format: u32) -> String {
    let bytes = [
        (format & 0xFF) as u8,
        ((format >> 8) & 0xFF) as u8,
        ((format >> 16) & 0xFF) as u8,
        ((format >> 24) & 0xFF) as u8,
    ];

    // Check whether bytes are printable
    if bytes.iter().all(|&b| b.is_ascii_graphic()) {
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        format!("{:08x}", format)
    }
}

// Handle wl_shm events
impl Dispatch<WlShm, UserData> for AppData {
    fn event(
        state: &mut Self,
        _shm: &WlShm,
        event: wl_shm::Event,
        data: &UserData,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
    ) {
        if let UserData::Shm { shm_index } = data {
            if let wl_shm::Event::Format { format } = event {
                state.add_shm_format(*shm_index, format);
            }
        }
    }
}
