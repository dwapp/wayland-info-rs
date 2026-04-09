use colored::Colorize;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::{AppData, GlobalInfo};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonOutput {
    generation_timestamp: u64,
    globals: Vec<GlobalNode>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalNode {
    pub interface: String,
    pub version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<serde_json::Value>,
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

    let mut globals = Vec::new();

    let iter = app_data
        .globals
        .iter()
        .filter(|g| protocol_filter.is_none_or(|p| g.interface == p));

    for global in iter {
        let info = get_protocol_info_json(app_data, &global.interface, global.name);

        globals.push(GlobalNode {
            interface: global.interface.clone(),
            version: global.version,
            info,
        });
    }

    if sort_output {
        globals.sort_by(|a, b| a.interface.cmp(&b.interface));
    }

    JsonOutput {
        generation_timestamp: timestamp_ms,
        globals,
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
                global.name.to_string().dimmed(),
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
                    println!(
                        "        capabilities: {}",
                        seat.capabilities.join(" ").cyan()
                    );
                }
                if let Some(rate) = seat.keyboard_repeat_rate {
                    println!(
                        "        keyboard repeat rate: {}",
                        rate.to_string().yellow()
                    );
                }
                if let Some(delay) = seat.keyboard_repeat_delay {
                    println!(
                        "        keyboard repeat delay: {}",
                        delay.to_string().yellow()
                    );
                }
            }
        }

        if global.interface == "wl_output" {
            if let Some(output) = app_data.outputs.iter().find(|o| o.name == global.name) {
                println!("        name: {}", output.output_name.yellow());
                if !output.description.is_empty() {
                    println!("        description: {}", output.description.cyan());
                }
                println!(
                    "        x: {}, y: {}, scale: {},",
                    output.x.to_string().yellow(),
                    output.y.to_string().yellow(),
                    output.scale.to_string().yellow()
                );
                println!(
                    "        physical_width: {} mm, physical_height: {} mm,",
                    output.physical_width.to_string().yellow(),
                    output.physical_height.to_string().yellow()
                );
                if !output.make.is_empty() {
                    println!(
                        "        make: '{}', model: '{}',",
                        output.make.green(),
                        output.model.green()
                    );
                }
                println!(
                    "        subpixel_orientation: {}, output_transform: {},",
                    output.subpixel_orientation.cyan(),
                    output.output_transform.cyan()
                );

                for mode in &output.modes {
                    println!(
                        "        mode:\n                width: {} px, height: {} px, refresh: {:.3} Hz,\n                flags: {}", mode.width.to_string().yellow(), mode.height.to_string().yellow(), (mode.refresh as f32 / 1000.0).to_string().yellow(), mode.flags.join(" ").cyan()
                    );
                }
            }
        }

        if global.interface == "wl_shm" {
            if let Some(shm) = app_data.shm_info.iter().find(|s| s.name == global.name) {
                for format in &shm.formats {
                    println!(
                        "        format: {} ({})",
                        format.fourcc.green(),
                        format.format.to_string().yellow()
                    );
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
                    println!("        path: {}", path.green());
                } else {
                    println!("        path: {}", "<unknown>".red());
                }

                if !device.connectors.is_empty() {
                    println!("        connectors:");
                    for connector in &device.connectors {
                        println!(
                            "                name: {}, description: {}, connector_id: {}",
                            connector.name.green(),
                            connector.description.cyan(),
                            connector.connector_id.to_string().yellow()
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
                        clock_id.to_string().yellow()
                    );
                } else {
                    println!("        presentation clock id: {}", "<unknown>".red());
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
                    println!("        Primary output: {}", "<unknown>".red());
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
                    println!("        {}", "xdg_output_v1".cyan());
                    println!(
                        "                output: {}",
                        output.output_id.to_string().yellow()
                    );
                    println!("                name: '{}'", output.name.green());
                    println!(
                        "                description: '{}'",
                        output.description.cyan()
                    );
                    println!(
                        "                logical_x: {}, logical_y: {}",
                        output.logical_x.to_string().yellow(),
                        output.logical_y.to_string().yellow()
                    );
                    println!(
                        "                logical_width: {}, logical_height: {}",
                        output.logical_width.to_string().yellow(),
                        output.logical_height.to_string().yellow()
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
                global.name.to_string().dimmed(),
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

fn get_protocol_info_json(
    app_data: &AppData,
    interface: &str,
    name: u32,
) -> Option<serde_json::Value> {
    match interface {
        "wl_seat" => app_data
            .seats
            .iter()
            .find(|s| s.name == name)
            .map(|s| serde_json::to_value(vec![s]).unwrap()),
        "wl_output" => app_data
            .outputs
            .iter()
            .find(|o| o.name == name)
            .map(|o| serde_json::to_value(vec![o]).unwrap()),
        "wl_shm" => app_data
            .shm_info
            .iter()
            .find(|s| s.name == name)
            .map(|s| serde_json::to_value(vec![s]).unwrap()),
        "wp_drm_lease_device_v1" => app_data
            .drm_lease_devices
            .iter()
            .find(|d| d.name == name)
            .map(|d| serde_json::to_value(vec![d]).unwrap()),
        "wp_presentation" => app_data
            .presentation_info
            .iter()
            .find(|p| p.name == name)
            .map(|p| serde_json::to_value(vec![p]).unwrap()),
        "treeland_output_manager_v1" => app_data
            .treeland_output_managers
            .iter()
            .find(|m| m.name == name)
            .map(|m| serde_json::to_value(vec![m]).unwrap()),
        "zxdg_output_manager_v1" => app_data
            .xdg_output_managers
            .iter()
            .find(|m| m.name == name)
            .map(|m| serde_json::to_value(vec![m]).unwrap()),
        _ => None,
    }
}
