use wayland_client::{
    protocol::{wl_output::WlOutput, wl_registry, wl_seat::WlSeat, wl_shm::WlShm},
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols::wp::presentation_time::client::wp_presentation::WpPresentation;
use wayland_protocols::xdg::xdg_output::zv1::client::zxdg_output_manager_v1::ZxdgOutputManagerV1;
use wayland_protocols_treeland::output_manager::v1::client::treeland_output_manager_v1::TreelandOutputManagerV1;

use crate::app::{AppData, UserData};

// Handle wl_registry events
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
            } else if interface == "treeland_output_manager_v1" {
                state.add_treeland_output_manager(name);
                let manager_index = state.treeland_output_managers.len() - 1;
                let manager = registry.bind::<TreelandOutputManagerV1, _, _>(
                    name,
                    version,
                    qh,
                    UserData::TreelandOutputManager { manager_index },
                );
                state.treeland_output_manager_objects.push(manager);
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
