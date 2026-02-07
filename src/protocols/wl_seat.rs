use serde::Serialize;
use wayland_client::{
    protocol::{wl_keyboard, wl_seat, wl_seat::WlSeat},
    Connection, Dispatch, QueueHandle, WEnum,
};

use crate::app::{AppData, UserData};

// Seat info structure
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatInfo {
    #[serde(skip_serializing)]
    pub(crate) name: u32,
    pub(crate) seat_name: String,
    pub(crate) capabilities: Vec<String>,
    pub(crate) keyboard_repeat_rate: Option<i32>,
    pub(crate) keyboard_repeat_delay: Option<i32>,
}

impl AppData {
    pub(crate) fn add_seat(&mut self, name: u32, seat_name: String) {
        self.seats.push(SeatInfo {
            name,
            seat_name,
            capabilities: Vec::new(),
            keyboard_repeat_rate: None,
            keyboard_repeat_delay: None,
        });
    }

    pub(crate) fn update_seat_capabilities(
        &mut self,
        seat_index: usize,
        capabilities: Vec<String>,
    ) {
        if let Some(seat) = self.seats.get_mut(seat_index) {
            seat.capabilities = capabilities;
        }
    }

    pub(crate) fn update_seat_keyboard_repeat(&mut self, seat_index: usize, rate: i32, delay: i32) {
        if let Some(seat) = self.seats.get_mut(seat_index) {
            seat.keyboard_repeat_rate = Some(rate);
            seat.keyboard_repeat_delay = Some(delay);
        }
    }
}

// Handle wl_seat events
impl Dispatch<WlSeat, UserData> for AppData {
    fn event(
        state: &mut Self,
        _seat: &WlSeat,
        event: wayland_client::protocol::wl_seat::Event,
        data: &UserData,
        _conn: &Connection,
        _qh: &QueueHandle<AppData>,
    ) {
        if let UserData::Seat { seat_index } = data {
            match event {
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
                wl_seat::Event::Name { name } => {
                    if let Some(seat) = state.seats.get_mut(*seat_index) {
                        seat.seat_name = name;
                    }
                }
                _ => {}
            }
        }
    }
}

// Handle wl_keyboard events
impl Dispatch<wl_keyboard::WlKeyboard, UserData> for AppData {
    fn event(
        state: &mut Self,
        _keyboard: &wl_keyboard::WlKeyboard,
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
