use colored::Colorize;
use serde::Serialize;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use wayland_client::{
    protocol::{
        wl_keyboard::{self, WlKeyboard},
        wl_output::{self, WlOutput},
        wl_registry,
        wl_seat::{self, WlSeat},
        wl_shm::{self, WlShm},
    },
    Connection, Dispatch, QueueHandle, WEnum,
};
use wayland_protocols::wp::presentation_time::client::wp_presentation::{self, WpPresentation};
use wayland_protocols::xdg::xdg_output::zv1::client::{
    zxdg_output_manager_v1::{self, ZxdgOutputManagerV1},
    zxdg_output_v1::{self, ZxdgOutputV1},
};

// 全局信息结构
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GlobalInfo {
    #[serde(skip_serializing)]
    name: u32,
    interface: String,
    version: u32,
}

// Seat 信息结构
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SeatInfo {
    #[serde(skip_serializing)]
    name: u32,
    seat_name: String,
    capabilities: Vec<String>,
    keyboard_repeat_rate: Option<i32>,
    keyboard_repeat_delay: Option<i32>,
}

// Output 信息结构
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OutputInfo {
    #[serde(skip_serializing)]
    name: u32,
    output_name: String,
    description: String,
    x: i32,
    y: i32,
    scale: i32,
    physical_width: i32,
    physical_height: i32,
    make: String,
    model: String,
    subpixel_orientation: String,
    output_transform: String,
    modes: Vec<OutputMode>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OutputMode {
    width: i32,
    height: i32,
    refresh: i32,
    flags: Vec<String>,
}

// SHM 信息结构
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShmInfo {
    #[serde(skip_serializing)]
    name: u32,
    formats: Vec<ShmFormat>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShmFormat {
    format: u32,
    fourcc: String,
}

// DRM Lease Device 信息结构
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DrmLeaseDeviceInfo {
    #[serde(skip_serializing)]
    name: u32,
    device_path: Option<String>,
    connectors: Vec<DrmLeaseConnectorInfo>,
}

// DRM Lease Connector 信息结构
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DrmLeaseConnectorInfo {
    name: String,
    description: String,
    connector_id: u32,
}

// Presentation 信息结构
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PresentationInfo {
    #[serde(skip_serializing)]
    name: u32,
    clock_id: Option<u32>,
}

// XDG Output Manager 信息结构
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct XdgOutputManagerInfo {
    #[serde(skip_serializing)]
    name: u32,
    outputs: Vec<XdgOutputInfo>,
}

// XDG Output 信息结构
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct XdgOutputInfo {
    #[serde(skip_serializing)]
    output_id: u32,
    name: String,
    description: String,
    logical_x: i32,
    logical_y: i32,
    logical_width: i32,
    logical_height: i32,
}

// Output Geometry 信息结构，用于减少函数参数
#[derive(Debug)]
struct OutputGeometry {
    x: i32,
    y: i32,
    physical_width: i32,
    physical_height: i32,
    subpixel: WEnum<wl_output::Subpixel>,
    transform: WEnum<wl_output::Transform>,
    make: String,
    model: String,
}

// 应用状态，用于收集所有信息
struct AppData {
    globals: Vec<GlobalInfo>,
    seats: Vec<SeatInfo>,
    outputs: Vec<OutputInfo>,
    shm_info: Vec<ShmInfo>,
    drm_lease_devices: Vec<DrmLeaseDeviceInfo>,
    presentation_info: Vec<PresentationInfo>,
    xdg_output_managers: Vec<XdgOutputManagerInfo>,
    seat_objects: Vec<WlSeat>,
    output_objects: Vec<WlOutput>,
    shm_objects: Vec<WlShm>,
    presentation_objects: Vec<WpPresentation>,
    xdg_output_manager_objects: Vec<ZxdgOutputManagerV1>,
    xdg_output_objects: Vec<ZxdgOutputV1>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonOutput {
    generation_timestamp: u64,
    globals: Vec<GlobalInfo>,
    seats: Vec<SeatInfo>,
    outputs: Vec<OutputInfo>,
    shm_info: Vec<ShmInfo>,
    drm_lease_devices: Vec<DrmLeaseDeviceInfo>,
    presentation_info: Vec<PresentationInfo>,
    xdg_output_managers: Vec<XdgOutputManagerInfo>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonOutputBasic {
    generation_timestamp: u64,
    globals: Vec<GlobalSummary>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GlobalSummary {
    interface: String,
    version: u32,
}

impl AppData {
    fn new() -> Self {
        Self {
            globals: Vec::new(),
            seats: Vec::new(),
            outputs: Vec::new(),
            shm_info: Vec::new(),
            drm_lease_devices: Vec::new(),
            presentation_info: Vec::new(),
            xdg_output_managers: Vec::new(),
            seat_objects: Vec::new(),
            output_objects: Vec::new(),
            shm_objects: Vec::new(),
            presentation_objects: Vec::new(),
            xdg_output_manager_objects: Vec::new(),
            xdg_output_objects: Vec::new(),
        }
    }

    fn add_global(&mut self, name: u32, interface: String, version: u32) {
        self.globals.push(GlobalInfo {
            name,
            interface,
            version,
        });
    }

    fn add_seat(&mut self, name: u32, seat_name: String) {
        self.seats.push(SeatInfo {
            name,
            seat_name,
            capabilities: Vec::new(),
            keyboard_repeat_rate: None,
            keyboard_repeat_delay: None,
        });
    }

    fn add_output(&mut self, name: u32, output_name: String) {
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

    fn add_shm(&mut self, name: u32) {
        self.shm_info.push(ShmInfo {
            name,
            formats: Vec::new(),
        });
    }

    fn add_shm_format(&mut self, shm_index: usize, format: u32) {
        if let Some(shm) = self.shm_info.get_mut(shm_index) {
            let fourcc = format_to_fourcc(format);
            shm.formats.push(ShmFormat { format, fourcc });
        }
    }

    fn add_drm_lease_device(&mut self, name: u32) {
        self.drm_lease_devices.push(DrmLeaseDeviceInfo {
            name,
            device_path: None,
            connectors: Vec::new(),
        });
    }

    #[allow(dead_code)]
    fn update_drm_lease_device_path(&mut self, device_index: usize, path: String) {
        if let Some(device) = self.drm_lease_devices.get_mut(device_index) {
            device.device_path = Some(path);
        }
    }

    #[allow(dead_code)]
    fn add_drm_lease_connector(
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

    fn add_presentation(&mut self, name: u32) {
        self.presentation_info.push(PresentationInfo {
            name,
            clock_id: None,
        });
    }

    fn update_presentation_clock_id(&mut self, presentation_index: usize, clock_id: u32) {
        if let Some(presentation) = self.presentation_info.get_mut(presentation_index) {
            presentation.clock_id = Some(clock_id);
        }
    }

    fn update_seat_capabilities(&mut self, seat_index: usize, capabilities: Vec<String>) {
        if let Some(seat) = self.seats.get_mut(seat_index) {
            seat.capabilities = capabilities;
        }
    }

    fn update_seat_keyboard_repeat(&mut self, seat_index: usize, rate: i32, delay: i32) {
        if let Some(seat) = self.seats.get_mut(seat_index) {
            seat.keyboard_repeat_rate = Some(rate);
            seat.keyboard_repeat_delay = Some(delay);
        }
    }

    fn update_output_geometry(&mut self, output_index: usize, geometry: OutputGeometry) {
        if let Some(output) = self.outputs.get_mut(output_index) {
            output.x = geometry.x;
            output.y = geometry.y;
            output.physical_width = geometry.physical_width;
            output.physical_height = geometry.physical_height;
            output.make = geometry.make;
            output.model = geometry.model;

            // 转换 subpixel 枚举为字符串
            output.subpixel_orientation = match geometry.subpixel {
                WEnum::Value(wl_output::Subpixel::Unknown) => "unknown".to_string(),
                WEnum::Value(wl_output::Subpixel::None) => "none".to_string(),
                WEnum::Value(wl_output::Subpixel::HorizontalRgb) => "horizontal_rgb".to_string(),
                WEnum::Value(wl_output::Subpixel::HorizontalBgr) => "horizontal_bgr".to_string(),
                WEnum::Value(wl_output::Subpixel::VerticalRgb) => "vertical_rgb".to_string(),
                WEnum::Value(wl_output::Subpixel::VerticalBgr) => "vertical_bgr".to_string(),
                _ => "unknown".to_string(),
            };

            // 转换 transform 枚举为字符串
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

    fn update_output_mode(
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

    fn update_output_scale(&mut self, output_index: usize, factor: i32) {
        if let Some(output) = self.outputs.get_mut(output_index) {
            output.scale = factor;
        }
    }

    fn update_output_name(&mut self, output_index: usize, name: String) {
        if let Some(output) = self.outputs.get_mut(output_index) {
            output.output_name = name;
        }
    }

    fn update_output_description(&mut self, output_index: usize, description: String) {
        if let Some(output) = self.outputs.get_mut(output_index) {
            output.description = description;
        }
    }

    fn add_xdg_output_manager(&mut self, name: u32) {
        self.xdg_output_managers.push(XdgOutputManagerInfo {
            name,
            outputs: Vec::new(),
        });
    }

    fn add_xdg_output(&mut self, manager_index: usize, output_id: u32) {
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

    fn update_xdg_output_name(&mut self, manager_index: usize, output_index: usize, name: String) {
        if let Some(manager) = self.xdg_output_managers.get_mut(manager_index) {
            if let Some(output) = manager.outputs.get_mut(output_index) {
                output.name = name;
            }
        }
    }

    fn update_xdg_output_description(
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

    fn update_xdg_output_logical_position(
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

    fn update_xdg_output_logical_size(
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

    fn to_json_output(&self, sort_output: bool) -> JsonOutput {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let mut globals = self.globals.clone();
        if sort_output {
            globals.sort_by(|a, b| a.interface.cmp(&b.interface));
        }

        JsonOutput {
            generation_timestamp: timestamp_ms,
            globals,
            seats: self.seats.clone(),
            outputs: self.outputs.clone(),
            shm_info: self.shm_info.clone(),
            drm_lease_devices: self.drm_lease_devices.clone(),
            presentation_info: self.presentation_info.clone(),
            xdg_output_managers: self.xdg_output_managers.clone(),
        }
    }

    fn to_json_basic(&self, sort_output: bool) -> JsonOutputBasic {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let mut globals: Vec<GlobalSummary> = self
            .globals
            .iter()
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

    fn print_all_info(&self, sort_output: bool) {
        println!("{}", "Wayland Global Interfaces:".bold().blue());
        let mut globals: Vec<&GlobalInfo> = self.globals.iter().collect();
        if sort_output {
            globals.sort_by(|a, b| a.interface.cmp(&b.interface));
        }
        for global in globals {
            // 接口类型用蓝色
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

            // 打印 seat 信息
            if global.interface == "wl_seat" {
                if let Some(seat) = self.seats.iter().find(|s| s.name == global.name) {
                    // seat 名用绿色
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

            // 打印 output 信息
            if global.interface == "wl_output" {
                if let Some(output) = self.outputs.iter().find(|o| o.name == global.name) {
                    // output 名用黄色
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

            // 打印 shm 信息
            if global.interface == "wl_shm" {
                if let Some(shm) = self.shm_info.iter().find(|s| s.name == global.name) {
                    for format in &shm.formats {
                        println!("        format: {} ({})", format.fourcc, format.format);
                    }
                }
            }

            // 打印 wp_drm_lease_device_v1 信息
            if global.interface == "wp_drm_lease_device_v1" {
                if let Some(device) = self
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

            // 打印 wp_presentation 信息
            if global.interface == "wp_presentation" {
                if let Some(presentation) = self
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

            // 打印 zxdg_output_manager_v1 信息
            if global.interface == "zxdg_output_manager_v1" {
                if let Some(manager) = self
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

    fn print_basic_info(&self, sort_output: bool) {
        println!("{}", "Wayland Global Interfaces:".bold().blue());
        let mut globals: Vec<&GlobalInfo> = self.globals.iter().collect();
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
        }
    }
}

// 通用用户数据类型
#[derive(Debug)]
#[allow(dead_code)]
enum UserData {
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
    XdgOutputManager {
        manager_index: usize,
    },
    XdgOutput {
        manager_index: usize,
        output_index: usize,
    },
}

// 将格式代码转换为 FOURCC 字符串
fn format_to_fourcc(format: u32) -> String {
    let bytes = [
        (format & 0xFF) as u8,
        ((format >> 8) & 0xFF) as u8,
        ((format >> 16) & 0xFF) as u8,
        ((format >> 24) & 0xFF) as u8,
    ];

    // 检查是否为可打印字符
    if bytes.iter().all(|&b| b.is_ascii_graphic()) {
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        format!("{:08x}", format)
    }
}

// 实现 wl_registry 的事件处理
impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _conn: &Connection,
        qh: &QueueHandle<AppData>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == "wl_seat" {
                state.add_seat(name, format!("seat{}", state.seats.len()));
                let seat_index = state.seats.len() - 1;
                let seat =
                    registry.bind::<WlSeat, _, _>(name, version, qh, UserData::Seat { seat_index });
                state.seat_objects.push(seat);
            } else if interface == "wl_output" {
                state.add_output(name, format!("output{}", state.outputs.len()));
                let output_index = state.outputs.len() - 1;
                let output = registry.bind::<WlOutput, _, _>(
                    name,
                    version,
                    qh,
                    UserData::Output { output_index },
                );
                state.output_objects.push(output);
            } else if interface == "wl_shm" {
                state.add_shm(name);
                let shm_index = state.shm_info.len() - 1;
                let shm =
                    registry.bind::<WlShm, _, _>(name, version, qh, UserData::Shm { shm_index });
                state.shm_objects.push(shm);
            } else if interface == "wp_drm_lease_device_v1" {
                state.add_drm_lease_device(name);
            } else if interface == "wp_presentation" {
                state.add_presentation(name);
                let presentation_index = state.presentation_info.len() - 1;
                let presentation = registry.bind::<WpPresentation, _, _>(
                    name,
                    version,
                    qh,
                    UserData::Presentation { presentation_index },
                );
                state.presentation_objects.push(presentation);
            } else if interface == "zxdg_output_manager_v1" {
                state.add_xdg_output_manager(name);
                let manager_index = state.xdg_output_managers.len() - 1;
                let manager = registry.bind::<ZxdgOutputManagerV1, _, _>(
                    name,
                    version,
                    qh,
                    UserData::XdgOutputManager { manager_index },
                );
                state.xdg_output_manager_objects.push(manager);
            }
            state.add_global(name, interface, version);
        }
    }
}

// 实现 wl_seat 的事件处理
impl Dispatch<WlSeat, UserData> for AppData {
    fn event(
        state: &mut Self,
        _seat: &WlSeat,
        event: wl_seat::Event,
        data: &UserData,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
    ) {
        if let UserData::Seat { seat_index } = data {
            match event {
                wl_seat::Event::Name { name } => {
                    if let Some(seat) = state.seats.get_mut(*seat_index) {
                        seat.seat_name = name;
                    }
                }
                wl_seat::Event::Capabilities { capabilities } => {
                    let mut caps = Vec::new();
                    match capabilities {
                        WEnum::Value(wl_seat::Capability::Pointer) => {
                            caps.push("pointer".to_string())
                        }
                        WEnum::Value(wl_seat::Capability::Keyboard) => {
                            caps.push("keyboard".to_string())
                        }
                        WEnum::Value(wl_seat::Capability::Touch) => caps.push("touch".to_string()),
                        _ => {}
                    }
                    state.update_seat_capabilities(*seat_index, caps);
                }
                _ => {}
            }
        }
    }
}

// 实现 wl_keyboard 的事件处理
impl Dispatch<WlKeyboard, UserData> for AppData {
    fn event(
        state: &mut Self,
        _keyboard: &WlKeyboard,
        event: wl_keyboard::Event,
        data: &UserData,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
    ) {
        if let UserData::Seat { seat_index } = data {
            if let wl_keyboard::Event::RepeatInfo { rate, delay } = event {
                state.update_seat_keyboard_repeat(*seat_index, rate, delay);
            }
        }
    }
}

// 实现 wl_output 的事件处理
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
                    transform,
                    make,
                    model,
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

// 实现 wl_shm 的事件处理
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
                let format_value: u32 = format.into();
                state.add_shm_format(*shm_index, format_value);
            }
        }
    }
}

// 实现 wp_presentation 的事件处理
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

// 实现 zxdg_output_manager_v1 的事件处理
impl Dispatch<ZxdgOutputManagerV1, UserData> for AppData {
    fn event(
        _state: &mut Self,
        _manager: &ZxdgOutputManagerV1,
        _event: zxdg_output_manager_v1::Event,
        _data: &UserData,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
    ) {
        // xdg_output objects are already created during binding
    }
}

// 实现 zxdg_output_v1 的事件处理
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
                zxdg_output_v1::Event::Done => {
                    // Output is done, no specific action needed
                }
                _ => {}
            }
        }
    }
}

fn print_help() {
    println!("wayland-info-rs options:");
    println!("    --json    Output JSON");
    println!("    --simple  Hide detailed protocol data (seats, outputs, shm, etc.)");
    println!("    --full    Include detailed protocol data (default)");
    println!("    --sort    Sort globals by interface (omit name field)");
    println!("    --help   Show this help");
}

fn main() {
    let mut json_output = false;
    let mut full_output = true;
    let mut sort_output = false;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--json" => json_output = true,
            "--full" => full_output = true,
            "--simple" => full_output = false,
            "--sort" => sort_output = true,
            "--help" | "-h" => {
                print_help();
                return;
            }
            _ => {}
        }
    }

    if env::var("WAYLAND_DISPLAY").is_err() {
        env::set_var("WAYLAND_DISPLAY", "wayland-0");
        eprintln!(
            "{}",
            "WAYLAND_DISPLAY was not set. Will try to use 'wayland-0'.".red()
        );
    }

    // 创建 Wayland 连接
    let conn = Connection::connect_to_env().unwrap();
    let display = conn.display();

    // 创建事件队列
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    // 创建应用状态
    let mut app_data = AppData::new();

    // 获取 registry
    let _registry = display.get_registry(&qh, ());

    // 执行 roundtrip 来收集所有全局对象信息
    event_queue.roundtrip(&mut app_data).unwrap();

    // 为每个 zxdg_output_manager_v1 创建对应的 xdg_output 对象
    let managers: Vec<_> = app_data.xdg_output_manager_objects.drain(..).collect();
    let outputs: Vec<_> = app_data.output_objects.drain(..).collect();

    let mut xdg_output_objects = Vec::new();
    let mut xdg_output_infos = vec![Vec::new(); managers.len()];

    for (manager_index, manager) in managers.iter().enumerate() {
        for (output_index, output) in outputs.iter().enumerate() {
            let xdg_output = manager.get_xdg_output(
                output,
                &qh,
                UserData::XdgOutput {
                    manager_index,
                    output_index,
                },
            );
            xdg_output_objects.push(xdg_output);
            xdg_output_infos[manager_index].push(output_index as u32);
        }
    }

    // 放回 app_data
    app_data.xdg_output_manager_objects = managers;
    app_data.output_objects = outputs;
    app_data.xdg_output_objects = xdg_output_objects;

    // 更新 manager 的输出信息
    for (manager_index, output_ids) in xdg_output_infos.into_iter().enumerate() {
        for output_id in output_ids {
            app_data.add_xdg_output(manager_index, output_id);
        }
    }

    // 为每个 seat 绑定键盘以获取重复信息
    let seat_objects: Vec<_> = app_data.seat_objects.drain(..).collect();
    for (index, seat) in seat_objects.iter().enumerate() {
        let seat_data = UserData::Seat { seat_index: index };
        let _keyboard = seat.get_keyboard(&qh, seat_data);
        event_queue.roundtrip(&mut app_data).unwrap();
    }

    // 多次 roundtrip 确保 xdg_output 事件能被处理
    for _ in 0..5 {
        event_queue.roundtrip(&mut app_data).unwrap();
    }

    // 统一输出所有信息
    if json_output {
        if full_output {
            let json_payload = app_data.to_json_output(sort_output);
            println!("{}", serde_json::to_string_pretty(&json_payload).unwrap());
        } else {
            let json_payload = app_data.to_json_basic(sort_output);
            println!("{}", serde_json::to_string_pretty(&json_payload).unwrap());
        }
    } else if full_output {
        app_data.print_all_info(sort_output);
    } else {
        app_data.print_basic_info(sort_output);
    }
}
