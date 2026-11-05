# AM03127 LED Panel Controller

This project provides a controller for AM03127 LED panels using an ESP32-C3 microcontroller. It communicates with the panel via RS232 and exposes a REST API through an HTTP server, along with a web interface for easy control.

## Features

- Control AM03127 LED panels via RS232 communication
- Display text with various animations and effects
- Create and manage schedules for automated content display
- Set and display the panel's internal clock
- RESTful API for programmatic control
- Web interface for easy management and OTA updates

## Hardware Requirements

- ESP32-C3 microcontroller
- AM03127 LED panel
- RS232 interface between ESP32-C3 and LED panel

## PCB Design

A custom PCB has been designed for this project. All PCB-related files, including schematics, board layouts, and manufacturing files, can be found in the [PCB](PCB/) folder.

## Software Architecture

The project is built with Rust and uses the following components:

- **Embassy**: Async runtime for embedded systems
- **ESP-HAL**: Hardware abstraction layer for ESP32 devices
- **PicoServe**: Lightweight HTTP server for embedded systems
- **Heapless**: Collections that don't require dynamic memory allocation

## Programs

The repository is a Cargo workspace plus the firmware, which is built separately for the ESP32-C3.

| Directory | What it is |
|---|---|
| [`am03127-firmware`](am03127-firmware/) | Firmware for the ESP32-C3: drives the panel over RS232, stores pages and schedules in flash, serves the REST API and the web UI, accepts OTA updates. |
| [`am03127-webui`](am03127-webui/) | Web UI written in Rust with [Leptos](https://leptos.dev) (WebAssembly). Edit pages and schedules, sync the clock, upload firmware. |
| [`am03127-server`](am03127-server/) | Server that consolidates all panels into one web UI and forwards each panel's API. |
| [`am03127-cli`](am03127-cli/) | Command line tool to control several panels at once. |
| [`am03127-client`](am03127-client/) | Shared HTTP client for the panel API and the `panel-config.toml` parser. Used by the CLI, the web UI and the server. |
| [`am03127-commands`](am03127-commands/) | Shared `no_std` models (pages, schedules, clock) and their AM03127 command encoding. Used by the firmware and everything else. |

### Firmware

The firmware connects to Wi-Fi, restores the stored pages and schedules on the panel and serves the API on port 80. The Wi-Fi credentials are compiled in:

```sh
export WIFI_SSID="MyNetwork" WIFI_PASS="secret"
just webui-firmware                     # build the web UI the firmware embeds
cd am03127-firmware && cargo run --release   # flash over USB (also writes the partition table)
```

For updates over Wi-Fi, `just save-image` builds the web UI and the release firmware into `firmware.bin`, and `just ota <panel-ip>` uploads it. The web UI's Device tab can upload it as well.

`GET /status` reports the build and the controller's health, shown in the web UI and by `am03127-cli status`:

- `version`: short git hash of the build, `-dirty` if the tree had uncommitted changes
- `uptime_secs`, `rssi` (Wi-Fi dBm), `heap_free` / `heap_used` (bytes)
- `panel_ok`: whether the panel answered the last command (`null` before the first one)

### Web UI

One Leptos crate, built in two variants:

- **Device** (default): served by the firmware for the panel it runs on.
- **Server** (`--features server`): shows a bar with all panels and their status; click a panel to edit it.

Both share the same tables and Device tab. Pages and schedules are edited in place; the last row of each table adds a new entry.

### Server

`am03127-server` serves the server variant of the web UI and forwards `/api/panel/<name>/…` to the panel's API, so the browser only talks to the server and the panels need no CORS.

```sh
am03127-server --config panel-config.toml --listen 0.0.0.0:8080 --web-root <built web UI>
```

`GET /api/panels` lists the configured panels.

### CLI

`am03127-cli` works on all (non-hidden) panels of `panel-config.toml`, or on the ones given with `--panels`:

```sh
am03127-cli status                                   # reachability, firmware version and health of each panel
am03127-cli info firmware.bin                        # build info of a firmware file
am03127-cli clock                                    # set the clocks to the local time
am03127-cli page A "Hello" --leading scroll_left     # send page A
am03127-cli schedule A A --from 2026-12-24 --to 2026-12-26   # show page A in that window
am03127-cli update firmware.bin --panels Kitchen     # OTA update one panel
am03127-cli --help                                   # all commands and options
```

### Panel configuration

The CLI and the server read the panels from a `panel-config.toml`:

```toml
[[panels]]
name = "Kitchen"
address = "192.168.1.23"   # host name or IP of the panel
floor = "Obergeschoss"     # optional, groups the panels in the server's web UI
model = "long"             # optional: "long" or "color"; omit for a standard panel

[[panels]]
name = "Spare"
address = "LED-8E-84.lan"
hidden = true              # skipped unless named with --panels
```

## Nix

The flake provides:

- Packages: `am03127-cli` (default), `am03127-server`, `am03127-webui` (server variant, built).
- NixOS modules:
  - `am03127-server`: runs the server with the web UI (`configFile`, `address`, `port`, `openFirewall`).
  - `am03127-clock`: syncs the panel clocks on a systemd timer.
  - `am03127-wednesday-schedule`: programs a weekly schedule slot.
- A dev shell with the toolchain, Trunk, espflash and just.

```nix
services.am03127-server = {
  enable = true;
  configFile = ./panel-config.toml;
  port = 80;
  openFirewall = true;
};
```

## Development

| Recipe | What it does |
|---|---|
| `just webui` | Web UI (device variant) with live reload against a mock panel (`am03127-webui/mock.py`). |
| `just webui-server` | Web UI (server variant) with `am03127-server` and two mock panels. |
| `just webui-firmware` | Builds the gzipped web UI the firmware embeds. |
| `just save-image` | Builds web UI and release firmware into `firmware.bin`. |
| `just ota <ip>` | Uploads `firmware.bin` to a panel. |

Both web UI recipes serve on <http://127.0.0.1:8080/>.

## API Documentation

The REST API documentation is available as [OpenAPI Specification](docs/openapi.yaml)
