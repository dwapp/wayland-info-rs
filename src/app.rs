use serde::Serialize;
use wayland_client::protocol::{wl_output::WlOutput, wl_seat::WlSeat, wl_shm::WlShm};
use wayland_protocols::wp::presentation_time::client::wp_presentation::WpPresentation;
use wayland_protocols::xdg::xdg_output::zv1::client::{
    zxdg_output_manager_v1::ZxdgOutputManagerV1, zxdg_output_v1::ZxdgOutputV1,
};
use wayland_protocols_treeland::output_manager::v1::client::treeland_output_manager_v1::TreelandOutputManagerV1;

// Global info structure
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalInfo {
    #[serde(skip_serializing)]
    pub(crate) name: u32,
    pub(crate) interface: String,
    pub(crate) version: u32,
}

// App state for collecting all info
pub struct AppData {
    pub(crate) globals: Vec<GlobalInfo>,
    pub(crate) seats: Vec<crate::protocols::wl_seat::SeatInfo>,
    pub(crate) outputs: Vec<crate::protocols::wl_output::OutputInfo>,
    pub(crate) shm_info: Vec<crate::protocols::wl_shm::ShmInfo>,
    pub(crate) drm_lease_devices: Vec<crate::protocols::wp_drm_lease_device::DrmLeaseDeviceInfo>,
    pub(crate) presentation_info: Vec<crate::protocols::wp_presentation::PresentationInfo>,
    pub(crate) treeland_output_managers:
        Vec<crate::protocols::treeland_output_manager::TreelandOutputManagerInfo>,
    pub(crate) xdg_output_managers: Vec<crate::protocols::xdg_output::XdgOutputManagerInfo>,
    pub(crate) seat_objects: Vec<WlSeat>,
    pub(crate) output_objects: Vec<WlOutput>,
    pub(crate) shm_objects: Vec<WlShm>,
    pub(crate) presentation_objects: Vec<WpPresentation>,
    pub(crate) treeland_output_manager_objects: Vec<TreelandOutputManagerV1>,
    pub(crate) xdg_output_manager_objects: Vec<ZxdgOutputManagerV1>,
    pub(crate) xdg_output_objects: Vec<ZxdgOutputV1>,
}

impl AppData {
    pub fn new() -> Self {
        Self {
            globals: Vec::new(),
            seats: Vec::new(),
            outputs: Vec::new(),
            shm_info: Vec::new(),
            drm_lease_devices: Vec::new(),
            presentation_info: Vec::new(),
            treeland_output_managers: Vec::new(),
            xdg_output_managers: Vec::new(),
            seat_objects: Vec::new(),
            output_objects: Vec::new(),
            shm_objects: Vec::new(),
            presentation_objects: Vec::new(),
            treeland_output_manager_objects: Vec::new(),
            xdg_output_manager_objects: Vec::new(),
            xdg_output_objects: Vec::new(),
        }
    }

    pub(crate) fn add_global(&mut self, name: u32, interface: String, version: u32) {
        self.globals.push(GlobalInfo {
            name,
            interface,
            version,
        });
    }
}

// Common user data type
#[derive(Debug)]
#[allow(dead_code)]
pub enum UserData {
    Seat {
        seat_index: usize,
    },
    Output {
        output_index: usize,
    },
    Shm {
        shm_index: usize,
    },
    Presentation {
        presentation_index: usize,
    },
    TreelandOutputManager {
        manager_index: usize,
    },
    XdgOutputManager {
        manager_index: usize,
    },
    XdgOutput {
        manager_index: usize,
        output_index: usize,
    },
}
