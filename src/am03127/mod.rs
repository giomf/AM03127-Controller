#![allow(dead_code)]

pub mod page_content;
pub mod realtime_clock;
use core::fmt::Write;

use heapless::String;

// Constant for string size
pub const STRING_SIZE: usize = 64;
pub const DEFAULT_ID: u8 = 1;

pub fn set_id(id: u8) -> String<STRING_SIZE> {
    let mut buffer = String::<STRING_SIZE>::new();
    write!(&mut buffer, "<ID><{:02X}><E>", id).unwrap();
    buffer
}

fn wrap_command(id: u8, payload: &str) -> String<STRING_SIZE> {
    let checksum = checksum(payload);
    let mut buffer = String::<STRING_SIZE>::new();
    write!(&mut buffer, "<ID{:02X}>{}{:02X}<E>", id, payload, checksum).unwrap();
    buffer
}

fn checksum(payload: &str) -> u8 {
    let mut check: u8 = 0;
    for character in payload.as_bytes() {
        check ^= character;
    }
    check
}
