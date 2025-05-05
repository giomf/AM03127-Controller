# AM03127 LED Panel Controller

This project provides a controller for AM03127 LED panels using an ESP32-C3 microcontroller. It communicates with the panel via RS232 and exposes a REST API through an HTTP server, along with a web interface for easy control.

## Features

- Control AM03127 LED panels via RS232 communication
- Display text with various animations and effects
- Create and manage schedules for automated content display
- Set and display the panel's internal clock
- RESTful API for programmatic control
- Web interface for easy management

## Hardware Requirements

- ESP32-C3 microcontroller
- AM03127 LED panel
- RS232 interface between ESP32-C3 and LED panel

## Software Architecture

The project is built with Rust and uses the following components:

- **Embassy**: Async runtime for embedded systems
- **ESP-HAL**: Hardware abstraction layer for ESP32 devices
- **PicoServe**: Lightweight HTTP server for embedded systems
- **Heapless**: Collections that don't require dynamic memory allocation

## API Documentation

The REST API documentation is available in OpenAPI format:

- [OpenAPI Specification](docs/openapi.yaml)
- [HTML Documentation](docs/openapi.html)

## API Endpoints

### Pages

- `GET /page/{pageId}` - Get a specific page
- `POST /page/{pageId}` - Create or update a page
- `DELETE /page/{pageId}` - Delete a page
- `GET /pages` - Get all pages

### Schedules

- `GET /schedule/{scheduleId}` - Get a specific schedule
- `POST /schedule/{scheduleId}` - Create or update a schedule
- `DELETE /schedule/{scheduleId}` - Delete a schedule
- `GET /schedules` - Get all schedules

### Clock

- `GET /clock` - Display clock on panel
- `POST /clock` - Set panel clock

## License

This project is open source and available under the [MIT License](LICENSE).
