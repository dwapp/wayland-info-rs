# wayland-info-rs

A Rust implementation of `wayland-info`, a tool for displaying detailed information about Wayland compositor interfaces and their capabilities.

## Description

`wayland-info-rs` is a command-line utility that connects to a Wayland compositor and displays comprehensive information about available global interfaces, including detailed information for specific interfaces like `wl_seat` and `wl_output`.

## Features

### Core Functionality
- **Global Interface Discovery**: Lists all available Wayland global interfaces with their names, interface types, and versions
- **Structured Output**: Collects all information before displaying, ensuring consistent and organized output
- **Colorized Output**: Uses colored text to highlight different types of information

### Detailed Interface Information

#### wl_seat
Displays comprehensive seat information including:
- Seat name (e.g., "seat0")
- Capabilities (pointer, keyboard, touch)
- Keyboard repeat rate and delay settings

Example output:
```
name: 10   interface: wl_seat                                       version: 9
        name: seat0
        capabilities: pointer keyboard
        keyboard repeat rate: 25
        keyboard repeat delay: 600
```

#### wl_output
Shows detailed display output information including:
- Output name and description
- Physical dimensions and position
- Manufacturer and model information
- Subpixel orientation and transform settings
- Available display modes with resolution, refresh rate, and flags

Example output:
```
name: 59   interface: wl_output                                     version: 4
        name: eDP-1
        description: BOE CQ eDP-1-0x0747
        x: 0, y: 0, scale: 1,
        physical_width: 344 mm, physical_height: 194 mm,
        make: 'BOE CQ', model: 'eDP-1-0x0747',
        subpixel_orientation: unknown, output_transform: normal,
        mode:
                width: 1920 px, height: 1080 px, refresh: 60.027 Hz,
                flags: current
```

## Installation

### Prerequisites
- Rust toolchain (cargo, rustc)
- Wayland development libraries
- A Wayland compositor running

### Building from Source
```bash
git clone <repository-url>
cd wayland-info-rs
cargo build --release
```

### Running
```bash
# Run directly with cargo
cargo run

# Or run the built binary
./target/release/wayland-info-rs
```

## Usage

Simply run the binary in a Wayland session:

```bash
wayland-info-rs
```

The tool will automatically:
1. Connect to the Wayland compositor
2. Discover all available global interfaces
3. Collect detailed information for supported interfaces
4. Display the information in a structured format

### Environment Variables
- `WAYLAND_DISPLAY`: If not set, defaults to "wayland-0"

## Technical Details

### Architecture
- Uses `wayland-client` crate for Wayland protocol communication
- Implements event-driven architecture with `Dispatch` traits
- Collects all information before displaying to ensure proper ordering

### Supported Protocols
- `wl_registry`: Global interface discovery
- `wl_seat`: Input device and seat information
- `wl_output`: Display output information
- `wl_keyboard`: Keyboard-specific information

### Event Handling
The application implements several `Dispatch` traits:
- `Dispatch<wl_registry::WlRegistry, ()>`: Handles global interface discovery
- `Dispatch<WlSeat, SeatData>`: Processes seat events
- `Dispatch<WlKeyboard, SeatData>`: Handles keyboard events
- `Dispatch<WlOutput, OutputData>`: Processes output events

## Output Format

The output follows a consistent format:
- Global interfaces are listed with name, interface type, and version
- Detailed information for specific interfaces follows immediately after their global entry
- Information is indented for better readability
- Color coding helps distinguish different types of data

## Comparison with Original C Implementation

This Rust implementation provides the same functionality as the original C `wayland-info` tool but with:
- Memory safety through Rust's ownership system
- Better error handling
- More maintainable code structure
- Type-safe protocol handling

## Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.

## License

This project is licensed under the same terms as the original `wayland-info` project.

## Dependencies

- `wayland-client`: Wayland client library
- `colored`: Terminal colorization
- Standard Rust libraries

## Development

To work on this project:

1. Clone the repository
2. Install Rust toolchain
3. Run `cargo check` to verify compilation
4. Run `cargo run` to test the application
5. Use `cargo test` to run tests (when implemented)

## Future Enhancements

Potential areas for improvement:
- Support for additional Wayland protocols
- JSON output format option
- Filtering capabilities
- More detailed error reporting
- Configuration file support