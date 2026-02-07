# Usage

## Quick start

Run inside a Wayland session:

```bash
wayland-info-rs
```

By default it prints detailed protocol information.

## Options

```text
--json    Output JSON
--full    Include detailed protocol data (default)
--simple  Hide detailed protocol data
--sort    Sort globals by interface (omit name field)
--protocol, -p <name>  Only show matching protocol
--help    Show help
```

## Examples

Detailed text output (default):

```bash
wayland-info-rs
```

Simple text output:

```bash
wayland-info-rs --simple
```

JSON output (detailed):

```bash
wayland-info-rs --json
```

JSON output (simple):

```bash
wayland-info-rs --json --simple
```

Sorted output without name field:

```bash
wayland-info-rs --sort
```

Filter output to a single protocol:

```bash
wayland-info-rs -p wl_seat
```

## Environment

- `WAYLAND_DISPLAY`: If not set, defaults to `wayland-0`.
