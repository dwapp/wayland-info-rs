mod app;
mod cli;
mod output;
mod protocols;

use colored::Colorize;
use std::env;
use wayland_client::Connection;

use crate::app::AppData;
use crate::cli::parse_args;
use crate::output::{print_all_info, print_basic_info, to_json_basic, to_json_output};

fn main() {
    let options = parse_args();

    if env::var("WAYLAND_DISPLAY").is_err() {
        env::set_var("WAYLAND_DISPLAY", "wayland-0");
        eprintln!(
            "{}",
            "WAYLAND_DISPLAY was not set. Will try to use 'wayland-0'.".red()
        );
    }

    let conn = Connection::connect_to_env().unwrap();
    let display = conn.display();

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    let mut app_data = AppData::new();

    let _registry = display.get_registry(&qh, ());

    event_queue.roundtrip(&mut app_data).unwrap();

    let managers: Vec<_> = app_data.xdg_output_manager_objects.drain(..).collect();
    let outputs: Vec<_> = app_data.output_objects.drain(..).collect();

    let mut xdg_output_objects = Vec::new();
    let mut xdg_output_infos = vec![Vec::new(); managers.len()];

    for (manager_index, manager) in managers.iter().enumerate() {
        for (output_index, output) in outputs.iter().enumerate() {
            let xdg_output = manager.get_xdg_output(
                output,
                &qh,
                app::UserData::XdgOutput {
                    manager_index,
                    output_index,
                },
            );
            xdg_output_objects.push(xdg_output);
            xdg_output_infos[manager_index].push(output_index as u32);
        }
    }

    app_data.xdg_output_manager_objects = managers;
    app_data.output_objects = outputs;
    app_data.xdg_output_objects = xdg_output_objects;

    for (manager_index, output_ids) in xdg_output_infos.into_iter().enumerate() {
        for output_id in output_ids {
            app_data.add_xdg_output(manager_index, output_id);
        }
    }

    let seat_objects: Vec<_> = app_data.seat_objects.drain(..).collect();
    for (index, seat) in seat_objects.iter().enumerate() {
        let seat_data = app::UserData::Seat { seat_index: index };
        let _keyboard = seat.get_keyboard(&qh, seat_data);
        event_queue.roundtrip(&mut app_data).unwrap();
    }

    for _ in 0..5 {
        event_queue.roundtrip(&mut app_data).unwrap();
    }

    if options.json_output {
        if options.full_output {
            let json_payload = to_json_output(
                &app_data,
                options.sort_output,
                options.protocol_filter.as_deref(),
            );
            println!("{}", serde_json::to_string_pretty(&json_payload).unwrap());
        } else {
            let json_payload = to_json_basic(
                &app_data,
                options.sort_output,
                options.protocol_filter.as_deref(),
            );
            println!("{}", serde_json::to_string_pretty(&json_payload).unwrap());
        }
    } else if options.full_output {
        print_all_info(
            &app_data,
            options.sort_output,
            options.protocol_filter.as_deref(),
        );
    } else {
        print_basic_info(
            &app_data,
            options.sort_output,
            options.protocol_filter.as_deref(),
        );
    }
}
