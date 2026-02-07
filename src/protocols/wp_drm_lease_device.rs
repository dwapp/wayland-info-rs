use serde::Serialize;

use crate::app::AppData;

// DRM lease device info structure
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DrmLeaseDeviceInfo {
    #[serde(skip_serializing)]
    pub(crate) name: u32,
    pub(crate) device_path: Option<String>,
    pub(crate) connectors: Vec<DrmLeaseConnectorInfo>,
}

// DRM lease connector info structure
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DrmLeaseConnectorInfo {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) connector_id: u32,
}

impl AppData {
    pub(crate) fn add_drm_lease_device(&mut self, name: u32) {
        self.drm_lease_devices.push(DrmLeaseDeviceInfo {
            name,
            device_path: None,
            connectors: Vec::new(),
        });
    }

    #[allow(dead_code)]
    pub(crate) fn update_drm_lease_device_path(&mut self, device_index: usize, path: String) {
        if let Some(device) = self.drm_lease_devices.get_mut(device_index) {
            device.device_path = Some(path);
        }
    }

    #[allow(dead_code)]
    pub(crate) fn add_drm_lease_connector(
        &mut self,
        device_index: usize,
        name: String,
        description: String,
        connector_id: u32,
    ) {
        if let Some(device) = self.drm_lease_devices.get_mut(device_index) {
            device.connectors.push(DrmLeaseConnectorInfo {
                name,
                description,
                connector_id,
            });
        }
    }
}
