#![allow(dead_code)]

pub mod delete;
pub mod page_content;
pub mod realtime_clock;
pub mod schedule;

use core::fmt::{Display, Write};
use heapless::String;

// Constants for string sizes and defaults
/// Maximum size for message strings
pub const MESSAGE_STRING_SIZE: usize = 16;
/// Maximum size for command strings
pub const COMMAND_STRING_SIZE: usize = 64;
/// Size of the checksum in bytes
pub const CHECKSUM_STRING_SIZE: usize = 2;
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
    fn command(&self, id: u8) -> String<COMMAND_STRING_SIZE> {
        let mut payload = String::<{ COMMAND_STRING_SIZE - CHECKSUM_STRING_SIZE }>::new();
        write!(payload, "{}", self).unwrap();
        let checksum = checksum(&payload);
        let mut buffer = String::<COMMAND_STRING_SIZE>::new();
        write!(&mut buffer, "<ID{:02X}>{}{:02X}<E>", id, payload, checksum).unwrap();
        buffer
    }
}

/// Creates a command to set the ID of the LED panel
///
/// # Arguments
/// * `id` - The ID to set for the panel
///
/// # Returns
/// * A string containing the formatted command
pub fn set_id(id: u8) -> String<COMMAND_STRING_SIZE> {
    let mut buffer = String::<COMMAND_STRING_SIZE>::new();
    write!(&mut buffer, "<ID><{:02X}><E>", id).unwrap();
    buffer
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
