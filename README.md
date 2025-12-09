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

## API Documentation

The REST API documentation is available as [OpenAPI Specification](docs/openapi.yaml)
