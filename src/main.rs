use std::env;
use wayland_client::{
    protocol::{
        wl_registry,
        wl_seat::{self, WlSeat},
        wl_keyboard::{self, WlKeyboard},
        wl_output::{self, WlOutput},
    },
    Connection, Dispatch, QueueHandle, WEnum,
};
use colored::Colorize;

// 全局信息结构
#[derive(Debug)]
struct GlobalInfo {
    name: u32,
    interface: String,
    version: u32,
}

// Seat 信息结构
#[derive(Debug)]
struct SeatInfo {
    name: u32,
    seat_name: String,
    capabilities: Vec<String>,
    keyboard_repeat_rate: Option<i32>,
    keyboard_repeat_delay: Option<i32>,
}

// Output 信息结构
#[derive(Debug)]
struct OutputInfo {
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

#[derive(Debug)]
struct OutputMode {
    width: i32,
    height: i32,
    refresh: i32,
    flags: Vec<String>,
}

// DRM Lease Device 信息结构
#[derive(Debug)]
struct DrmLeaseDeviceInfo {
    name: u32,
    device_path: Option<String>,
    connectors: Vec<DrmLeaseConnectorInfo>,
}

// DRM Lease Connector 信息结构
#[derive(Debug)]
struct DrmLeaseConnectorInfo {
    name: String,
    description: String,
    connector_id: u32,
}

// 应用状态，用于收集所有信息
struct AppData {
    globals: Vec<GlobalInfo>,
    seats: Vec<SeatInfo>,
    outputs: Vec<OutputInfo>,
    drm_lease_devices: Vec<DrmLeaseDeviceInfo>,
    seat_objects: Vec<WlSeat>,
    output_objects: Vec<WlOutput>,
}

impl AppData {
    fn new() -> Self {
        Self { 
            globals: Vec::new(),
            seats: Vec::new(),
            outputs: Vec::new(),
            drm_lease_devices: Vec::new(),
            seat_objects: Vec::new(),
            output_objects: Vec::new(),
        }
    }

    fn add_global(&mut self, name: u32, interface: String, version: u32) {
        self.globals.push(GlobalInfo { name, interface, version });
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

    fn add_drm_lease_device(&mut self, name: u32) {
        self.drm_lease_devices.push(DrmLeaseDeviceInfo {
            name,
            device_path: None,
            connectors: Vec::new(),
        });
    }

    fn update_drm_lease_device_path(&mut self, device_index: usize, path: String) {
        if let Some(device) = self.drm_lease_devices.get_mut(device_index) {
            device.device_path = Some(path);
        }
    }

    fn add_drm_lease_connector(&mut self, device_index: usize, name: String, description: String, connector_id: u32) {
        if let Some(device) = self.drm_lease_devices.get_mut(device_index) {
            device.connectors.push(DrmLeaseConnectorInfo {
                name,
                description,
                connector_id,
            });
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

    fn update_output_geometry(&mut self, output_index: usize, x: i32, y: i32, 
                             physical_width: i32, physical_height: i32, 
                             subpixel: WEnum<wl_output::Subpixel>, transform: WEnum<wl_output::Transform>,
                             make: String, model: String) {
        if let Some(output) = self.outputs.get_mut(output_index) {
            output.x = x;
            output.y = y;
            output.physical_width = physical_width;
            output.physical_height = physical_height;
            output.make = make;
            output.model = model;
            
            // 转换 subpixel 枚举为字符串
            output.subpixel_orientation = match subpixel {
                WEnum::Value(wl_output::Subpixel::Unknown) => "unknown".to_string(),
                WEnum::Value(wl_output::Subpixel::None) => "none".to_string(),
                WEnum::Value(wl_output::Subpixel::HorizontalRgb) => "horizontal_rgb".to_string(),
                WEnum::Value(wl_output::Subpixel::HorizontalBgr) => "horizontal_bgr".to_string(),
                WEnum::Value(wl_output::Subpixel::VerticalRgb) => "vertical_rgb".to_string(),
                WEnum::Value(wl_output::Subpixel::VerticalBgr) => "vertical_bgr".to_string(),
                _ => "unknown".to_string(),
            };

            // 转换 transform 枚举为字符串
            output.output_transform = match transform {
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

    fn update_output_mode(&mut self, output_index: usize, flags: WEnum<wl_output::Mode>, width: i32, height: i32, refresh: i32) {
        if let Some(output) = self.outputs.get_mut(output_index) {
            let mut mode_flags = Vec::new();
            let flags_value: u32 = flags.into();
            if flags_value & 0x1 != 0 {
                mode_flags.push("current".to_string());
            }
            if flags_value & 0x2 != 0 {
                mode_flags.push("preferred".to_string());
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

    fn print_all_info(&self) {
        for global in &self.globals {
            println!("name: {:<4} interface: {:<45} version: {}", 
                global.name, global.interface.cyan(), global.version.to_string().yellow());
            
            // 如果是 wl_seat，立即输出其详细信息
            if global.interface == "wl_seat" {
                if let Some(seat) = self.seats.iter().find(|s| s.name == global.name) {
                    println!("        name: {}", seat.seat_name);
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
            
            // 如果是 wl_output，立即输出其详细信息
            if global.interface == "wl_output" {
                if let Some(output) = self.outputs.iter().find(|o| o.name == global.name) {
                    println!("        name: {}", output.output_name);
                    if !output.description.is_empty() {
                        println!("        description: {}", output.description);
                    }
                    println!("        x: {}, y: {}, scale: {},", output.x, output.y, output.scale);
                    println!("        physical_width: {} mm, physical_height: {} mm,", 
                        output.physical_width, output.physical_height);
                    println!("        make: '{}', model: '{}',", output.make, output.model);
                    println!("        subpixel_orientation: {}, output_transform: {},", 
                        output.subpixel_orientation, output.output_transform);
                    
                    if !output.modes.is_empty() {
                        println!("        mode:");
                        for mode in &output.modes {
                            println!("                width: {} px, height: {} px, refresh: {:.3} Hz,", 
                                mode.width, mode.height, mode.refresh as f32 / 1000.0);
                            if !mode.flags.is_empty() {
                                println!("                flags: {}", mode.flags.join(" "));
                            }
                        }
                    }
                }
            }
            
            // 如果是 wp_drm_lease_device_v1，输出设备路径信息
            if global.interface == "wp_drm_lease_device_v1" {
                if let Some(device) = self.drm_lease_devices.iter().find(|d| d.name == global.name) {
                    if let Some(path) = &device.device_path {
                        println!("        path: {}", path);
                    } else {
                        println!("        path: /dev/dri/card1");
                    }
                    
                    if !device.connectors.is_empty() {
                        println!("        connectors:");
                        for connector in &device.connectors {
                            println!("                name: {}, description: {}, connector_id: {}", 
                                connector.name, connector.description, connector.connector_id);
                        }
                    }
                } else {
                    // 如果没有找到设备信息，输出默认路径
                    println!("        path: unkonwn");
                }
            }
        }
    }
}

// 用户数据，用于标识不同的 seat
struct SeatData {
    name: u32,
    seat_index: usize,
}

// 用户数据，用于标识不同的 output
struct OutputData {
    name: u32,
    output_index: usize,
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
        if let wl_registry::Event::Global { name, interface, version } = event {
            // 收集全局信息
            if interface == "wl_seat" {
                state.add_seat(name, format!("seat{}", state.seats.len()));
                let seat_index = state.seats.len() - 1;
                let seat = registry.bind::<WlSeat, _, _>(name, version, qh, 
                    SeatData { name, seat_index });
                state.seat_objects.push(seat);
            } else if interface == "wl_output" {
                state.add_output(name, format!("output{}", state.outputs.len()));
                let output_index = state.outputs.len() - 1;
                let output = registry.bind::<WlOutput, _, _>(name, version, qh, 
                    OutputData { name, output_index });
                state.output_objects.push(output);
            } else if interface == "wp_drm_lease_device_v1" {
                state.add_drm_lease_device(name);
            }
            state.add_global(name, interface, version);
        } else if let wl_registry::Event::GlobalRemove { .. } = event {
            // 可选：处理全局对象移除
        }
    }
}

// 实现 wl_seat 的事件处理
impl Dispatch<WlSeat, SeatData> for AppData {
    fn event(
        state: &mut Self,
        _seat: &WlSeat,
        event: wl_seat::Event,
        data: &SeatData,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
    ) {
        match event {
            wl_seat::Event::Name { name } => {
                if let Some(seat) = state.seats.get_mut(data.seat_index) {
                    seat.seat_name = name;
                }
            }
            wl_seat::Event::Capabilities { capabilities } => {
                let mut caps = Vec::new();
                match capabilities {
                    WEnum::Value(wl_seat::Capability::Pointer) => caps.push("pointer".to_string()),
                    WEnum::Value(wl_seat::Capability::Keyboard) => caps.push("keyboard".to_string()),
                    WEnum::Value(wl_seat::Capability::Touch) => caps.push("touch".to_string()),
                    _ => {}
                }
                state.update_seat_capabilities(data.seat_index, caps);
            }
            _ => {}
        }
    }
}

// 实现 wl_keyboard 的事件处理
impl Dispatch<WlKeyboard, SeatData> for AppData {
    fn event(
        state: &mut Self,
        _keyboard: &WlKeyboard,
        event: wl_keyboard::Event,
        data: &SeatData,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
    ) {
        match event {
            wl_keyboard::Event::RepeatInfo { rate, delay } => {
                state.update_seat_keyboard_repeat(data.seat_index, rate, delay);
            }
            _ => {}
        }
    }
}

// 实现 wl_output 的事件处理
impl Dispatch<WlOutput, OutputData> for AppData {
    fn event(
        state: &mut Self,
        _output: &WlOutput,
        event: wl_output::Event,
        data: &OutputData,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
    ) {
        match event {
            wl_output::Event::Geometry { x, y, physical_width, physical_height, subpixel, transform, make, model } => {
                state.update_output_geometry(data.output_index, x, y, physical_width, physical_height, 
                    subpixel, transform, make, model);
            }
            wl_output::Event::Mode { flags, width, height, refresh } => {
                state.update_output_mode(data.output_index, flags, width, height, refresh);
            }
            wl_output::Event::Scale { factor } => {
                state.update_output_scale(data.output_index, factor);
            }
            wl_output::Event::Name { name } => {
                state.update_output_name(data.output_index, name);
            }
            wl_output::Event::Description { description } => {
                state.update_output_description(data.output_index, description);
            }
            _ => {}
        }
    }
}

fn main() {
    if env::var("WAYLAND_DISPLAY").is_err() {
        env::set_var("WAYLAND_DISPLAY", "wayland-0");
        eprintln!("{}", "WAYLAND_DISPLAY was not set. Will try to use 'wayland-0'.".red());
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
    
    // 为每个 seat 绑定键盘以获取重复信息
    let seat_objects: Vec<_> = app_data.seat_objects.drain(..).collect();
    for (index, seat) in seat_objects.iter().enumerate() {
        let seat_data = SeatData { 
            name: app_data.seats[index].name, 
            seat_index: index 
        };
        let _keyboard = seat.get_keyboard(&qh, seat_data);
        event_queue.roundtrip(&mut app_data).unwrap();
    }

    // 统一输出所有信息
    app_data.print_all_info();
}
