use std::env;
use wayland_client::{protocol::wl_registry, Connection, Dispatch, QueueHandle};
use colored::Colorize;

// This struct represents the state of our app. This simple app does not
// need any state, but this type still supports the `Dispatch` implementations.
struct AppData;

// Implement `Dispatch<WlRegistry, ()> for our state. This provides the logic
// to be able to process events for the wl_registry interface.
//
// The second type parameter is the user-data of our implementation. It is a
// mechanism that allows you to associate a value to each particular Wayland
// object, and allow different dispatching logic depending on the type of the
// associated value.
//
// In this example, we just use () as we don't have any value to associate. See
// the `Dispatch` documentation for more details about this.
impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        _state: &mut Self,
        _: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        // When receiving events from the wl_registry, we are only interested in the
        // `global` event, which signals a new available global.
        // When receiving this event, we just print its characteristics in this example.
        if let wl_registry::Event::Global { name, interface, version } = event {
            println!("name: {:<4} interface: {:<45} version: {}", name, interface.cyan(), version.to_string().yellow());
        }
    }
}

fn main() {
    if env::var("WAYLAND_DISPLAY").is_err() {
        env::set_var("WAYLAND_DISPLAY", "wayland-0");
        eprintln!("{}", "WAYLAND_DISPLAY was not set. Will try te use 'wayland-0'.".red());
    }
    // Create a Wayland connection by connecting to the server through the
    // environment-provided configuration.
    let conn = Connection::connect_to_env().unwrap();

    // Retrieve the WlDisplay Wayland object from the connection. This object is
    // the starting point of any Wayland program, from which all other objects will
    // be created.
    let display = conn.display();

    // Create an event queue for our event processing
    let mut event_queue = conn.new_event_queue();
    // And get its handle to associate new objects to it
    let qh = event_queue.handle();

    // Create a wl_registry object by sending the wl_display.get_registry request.
    // This method takes two arguments: a handle to the queue that the newly created
    // wl_registry will be assigned to, and the user-data that should be associated
    // with this registry (here it is () as we don't need user-data).
    let _registry = display.get_registry(&qh, ());

    // To actually receive the events, we invoke the `roundtrip` method. This method
    // is special and you will generally only invoke it during the setup of your program:
    // it will block until the server has received and processed all the messages you've
    // sent up to now.
    //
    // In our case, that means it'll block until the server has received our
    // wl_display.get_registry request, and as a reaction has sent us a batch of
    // wl_registry.global events.
    //
    // `roundtrip` will then empty the internal buffer of the queue it has been invoked
    // on, and thus invoke our `Dispatch` implementation that prints the list of advertised
    // globals.
    event_queue.roundtrip(&mut AppData).unwrap();
}
