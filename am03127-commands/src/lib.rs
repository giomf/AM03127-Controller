#![no_std]
#![allow(dead_code)]

pub mod delete;
pub mod formatting;
pub mod page;
pub mod realtime_clock;
pub mod schedule;

extern crate alloc;
use alloc::{
    format,
    string::{String, ToString},
};
use core::fmt::Display;

// Constants for string sizes and defaults
/// Default page ID
pub const DEFAULT_PAGE: char = 'A';
/// Default line number
pub const DEFAULT_LINE: u8 = 1;
/// Default schedule ID
pub const DEFAULT_SCHEDULE: char = 'A';

/// Trait for types that can be converted to AM03127 panel commands
///
/// This trait is implemented by types that represent commands for the LED panel.
/// It provides a method to convert the command to a string with the proper format,
/// including panel ID and checksum.
pub trait CommandAble: Display {
    /// Converts the command to a string with the proper format for the LED panel
    ///
    /// # Arguments
    /// * `id` - The ID of the panel to send the command to
    ///
    /// # Returns
    /// * A string containing the formatted command
    fn command(&self, id: u8) -> String {
        let payload = self.to_string();
        let checksum = checksum(&payload);
        format!("<ID{:02X}>{}{:02X}<E>", id, payload, checksum)
    }
}

/// Creates a command to set the ID of the LED panel
///
/// # Arguments
/// * `id` - The ID to set for the panel
///
/// # Returns
/// * A string containing the formatted command
pub fn set_id(id: u8) -> String {
    format!("<ID><{:02X}><E>", id)
}

/// Calculates the checksum for a command payload
///
/// The checksum is calculated by XORing all bytes in the payload.
///
/// # Arguments
/// * `payload` - The command payload
///
/// # Returns
/// * The calculated checksum
fn checksum(payload: &str) -> u8 {
    let mut check: u8 = 0;
    for character in payload.as_bytes() {
        check ^= character;
    }
    check
}
