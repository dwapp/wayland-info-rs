use std::env;
use wayland_client::{
    protocol::{
        wl_registry,
        wl_seat::{self, WlSeat},
        wl_keyboard::{self, WlKeyboard},
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

// 应用状态，用于收集所有信息
struct AppData {
    globals: Vec<GlobalInfo>,
    seats: Vec<SeatInfo>,
    seat_objects: Vec<WlSeat>,
}

impl AppData {
    fn new() -> Self {
        Self { 
            globals: Vec::new(),
            seats: Vec::new(),
            seat_objects: Vec::new(),
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
        }
    }
}

// 用户数据，用于标识不同的 seat
struct SeatData {
    name: u32,
    seat_index: usize,
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
