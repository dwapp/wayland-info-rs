use colored::Colorize;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::{AppData, GlobalInfo};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonOutput {
    generation_timestamp: u64,
    globals: Vec<GlobalInfo>,
    seats: Vec<crate::protocols::wl_seat::SeatInfo>,
    outputs: Vec<crate::protocols::wl_output::OutputInfo>,
    shm_info: Vec<crate::protocols::wl_shm::ShmInfo>,
    drm_lease_devices: Vec<crate::protocols::wp_drm_lease_device::DrmLeaseDeviceInfo>,
    presentation_info: Vec<crate::protocols::wp_presentation::PresentationInfo>,
    treeland_output_managers:
        Vec<crate::protocols::treeland_output_manager::TreelandOutputManagerInfo>,
    xdg_output_managers: Vec<crate::protocols::xdg_output::XdgOutputManagerInfo>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonOutputBasic {
    generation_timestamp: u64,
    globals: Vec<GlobalSummary>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSummary {
    interface: String,
    version: u32,
}

pub fn to_json_output(
    app_data: &AppData,
    sort_output: bool,
    protocol_filter: Option<&str>,
) -> JsonOutput {
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let mut globals = app_data.globals.clone();
    if let Some(protocol) = protocol_filter {
        globals.retain(|g| g.interface == protocol);
    }
    if sort_output {
        globals.sort_by(|a, b| a.interface.cmp(&b.interface));
    }

    if protocol_filter.is_none() {
        return JsonOutput {
            generation_timestamp: timestamp_ms,
            globals,
            seats: app_data.seats.clone(),
            outputs: app_data.outputs.clone(),
            shm_info: app_data.shm_info.clone(),
            drm_lease_devices: app_data.drm_lease_devices.clone(),
            presentation_info: app_data.presentation_info.clone(),
            treeland_output_managers: app_data.treeland_output_managers.clone(),
            xdg_output_managers: app_data.xdg_output_managers.clone(),
        };
    }

    JsonOutput {
        generation_timestamp: timestamp_ms,
        globals,
        seats: if protocol_filter == Some("wl_seat") {
            app_data.seats.clone()
        } else {
            Vec::new()
        },
        outputs: if protocol_filter == Some("wl_output") {
            app_data.outputs.clone()
        } else {
            Vec::new()
        },
        shm_info: if protocol_filter == Some("wl_shm") {
            app_data.shm_info.clone()
        } else {
            Vec::new()
        },
        drm_lease_devices: if protocol_filter == Some("wp_drm_lease_device_v1") {
            app_data.drm_lease_devices.clone()
        } else {
            Vec::new()
        },
        presentation_info: if protocol_filter == Some("wp_presentation") {
            app_data.presentation_info.clone()
        } else {
            Vec::new()
        },
        treeland_output_managers: if protocol_filter == Some("treeland_output_manager_v1") {
            app_data.treeland_output_managers.clone()
        } else {
            Vec::new()
        },
        xdg_output_managers: if protocol_filter == Some("zxdg_output_manager_v1") {
            app_data.xdg_output_managers.clone()
        } else {
            Vec::new()
        },
    }
}

pub fn to_json_basic(
    app_data: &AppData,
    sort_output: bool,
    protocol_filter: Option<&str>,
) -> JsonOutputBasic {
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let mut globals: Vec<GlobalSummary> = app_data
        .globals
        .iter()
        .filter(|g| protocol_filter.is_none_or(|p| g.interface == p))
        .map(|g| GlobalSummary {
            interface: g.interface.clone(),
            version: g.version,
        })
        .collect();
    if sort_output {
        globals.sort_by(|a, b| a.interface.cmp(&b.interface));
    }

    JsonOutputBasic {
        generation_timestamp: timestamp_ms,
        globals,
    }
}

pub fn print_all_info(app_data: &AppData, sort_output: bool, protocol_filter: Option<&str>) {
    if let Some(protocol) = protocol_filter {
        if !app_data.globals.iter().any(|g| g.interface == protocol) {
            println!("{} {}", "Protocol not supported:".red(), protocol);
            return;
        }
    }

    println!("{}", "Wayland Global Interfaces:".bold().blue());
    let mut globals: Vec<&GlobalInfo> = app_data
        .globals
        .iter()
        .filter(|g| protocol_filter.is_none_or(|p| g.interface == p))
        .collect();
    if sort_output {
        globals.sort_by(|a, b| a.interface.cmp(&b.interface));
    }
    for global in globals {
        if sort_output {
            println!(
                "interface: {:<45} version: {}",
                global.interface.blue(),
                global.version.to_string().yellow()
            );
        } else {
            println!(
                "name: {:<4} interface: {:<45} version: {}",
                global.name,
                global.interface.blue(),
                global.version.to_string().yellow()
            );
        }

        if let Some(protocol) = protocol_filter {
            if !protocol_has_details(protocol) {
                println!(
                    "{}",
                    "        [Info] wayland-info-rs does not implement details for this protocol"
                        .yellow()
                );
                continue;
            }
        }

        if global.interface == "wl_seat" {
            if let Some(seat) = app_data.seats.iter().find(|s| s.name == global.name) {
                println!("        name: {}", seat.seat_name.green());
                if !seat.capabilities.is_empty() {
                    println!("        capabilities: {}", seat.capabilities.join(" "));
                }
                if let Some(rate) = seat.keyboard_repeat_rate {
                    println!("        keyboard repeat rate: {}", rate);
                }
                if let Some(delay) = seat.keyboard_repeat_delay {
                    println!("        keyboard repeat delay: {}", delay);
                }
            }
        }

        if global.interface == "wl_output" {
            if let Some(output) = app_data.outputs.iter().find(|o| o.name == global.name) {
                println!("        name: {}", output.output_name.yellow());
                if !output.description.is_empty() {
                    println!("        description: {}", output.description);
                }
                println!(
                    "        x: {}, y: {}, scale: {},",
                    output.x, output.y, output.scale
                );
                println!(
                    "        physical_width: {} mm, physical_height: {} mm,",
                    output.physical_width, output.physical_height
                );
                if !output.make.is_empty() {
                    println!(
                        "        make: '{}', model: '{}',",
                        output.make, output.model
                    );
                }
                println!(
                    "        subpixel_orientation: {}, output_transform: {},",
                    output.subpixel_orientation, output.output_transform
                );

                for mode in &output.modes {
                    println!(
                        "        mode:\n                width: {} px, height: {} px, refresh: {:.3} Hz,\n                flags: {}",
                        mode.width,
                        mode.height,
                        mode.refresh as f32 / 1000.0,
                        mode.flags.join(" ")
                    );
                }
            }
        }

        if global.interface == "wl_shm" {
            if let Some(shm) = app_data.shm_info.iter().find(|s| s.name == global.name) {
                for format in &shm.formats {
                    println!("        format: {} ({})", format.fourcc, format.format);
                }
            }
        }

        if global.interface == "wp_drm_lease_device_v1" {
            if let Some(device) = app_data
                .drm_lease_devices
                .iter()
                .find(|d| d.name == global.name)
            {
                if let Some(path) = &device.device_path {
                    println!("        path: {}", path);
                } else {
                    println!("        path: /dev/dri/card1");
                }

                if !device.connectors.is_empty() {
                    println!("        connectors:");
                    for connector in &device.connectors {
                        println!(
                            "                name: {}, description: {}, connector_id: {}",
                            connector.name, connector.description, connector.connector_id
                        );
                    }
                }
            } else {
                println!(
                    "{}",
                    "        [Warning] DRM Lease device info not found!".red()
                );
            }
        }

        if global.interface == "wp_presentation" {
            if let Some(presentation) = app_data
                .presentation_info
                .iter()
                .find(|p| p.name == global.name)
            {
                if let Some(clock_id) = presentation.clock_id {
                    println!(
                        "        presentation clock id: {} (CLOCK_MONOTONIC)",
                        clock_id
                    );
                } else {
                    println!("        presentation clock id: 1 (CLOCK_MONOTONIC)");
                }
            } else {
                println!("{}", "        [Warning] Presentation info not found!".red());
            }
        }

        if global.interface == "treeland_output_manager_v1" {
            if let Some(manager) = app_data
                .treeland_output_managers
                .iter()
                .find(|m| m.name == global.name)
            {
                if let Some(primary) = &manager.primary_output {
                    println!("        Primary output: {}", primary.yellow());
                } else {
                    println!("{}", "        Primary output: <unknown>".red());
                }
            }
        }

        if global.interface == "zxdg_output_manager_v1" {
            if let Some(manager) = app_data
                .xdg_output_managers
                .iter()
                .find(|m| m.name == global.name)
            {
                for output in &manager.outputs {
                    println!("        xdg_output_v1");
                    println!("                output: {}", output.output_id);
                    println!("                name: '{}'", output.name);
                    println!("                description: '{}'", output.description);
                    println!(
                        "                logical_x: {}, logical_y: {}",
                        output.logical_x, output.logical_y
                    );
                    println!(
                        "                logical_width: {}, logical_height: {}",
                        output.logical_width, output.logical_height
                    );
                }
            }
        }
    }
}

pub fn print_basic_info(app_data: &AppData, sort_output: bool, protocol_filter: Option<&str>) {
    if let Some(protocol) = protocol_filter {
        if !app_data.globals.iter().any(|g| g.interface == protocol) {
            println!("{} {}", "Protocol not supported:".red(), protocol);
            return;
        }
    }

    println!("{}", "Wayland Global Interfaces:".bold().blue());
    let mut globals: Vec<&GlobalInfo> = app_data
        .globals
        .iter()
        .filter(|g| protocol_filter.is_none_or(|p| g.interface == p))
        .collect();
    if sort_output {
        globals.sort_by(|a, b| a.interface.cmp(&b.interface));
    }
    for global in globals {
        if sort_output {
            println!(
                "interface: {:<45} version: {}",
                global.interface.blue(),
                global.version.to_string().yellow()
            );
        } else {
            println!(
                "name: {:<4} interface: {:<45} version: {}",
                global.name,
                global.interface.blue(),
                global.version.to_string().yellow()
            );
        }

        if let Some(protocol) = protocol_filter {
            if !protocol_has_details(protocol) {
                println!(
                    "{}",
                    "        [Info] wayland-info-rs does not implement details for this protocol"
                        .yellow()
                );
            }
        }
    }
}

fn protocol_has_details(protocol: &str) -> bool {
    matches!(
        protocol,
        "wl_seat"
            | "wl_output"
            | "wl_shm"
            | "wp_drm_lease_device_v1"
            | "wp_presentation"
            | "treeland_output_manager_v1"
            | "zxdg_output_manager_v1"
    )
}
