#![allow(dead_code)]

pub mod delete;
pub mod page_content;
pub mod realtime_clock;
pub mod schedule;

use core::fmt::{Display, Write};
use heapless::String;

// Constant for string size
pub const MESSAGE_STRING_SIZE: usize = 16;
pub const COMMAND_STRING_SIZE: usize = 64;
pub const CHECKSUM_STRING_SIZE: usize = 2;
pub const DEFAULT_PAGE: char = 'A';
pub const DEFAULT_LINE: u8 = 1;
pub const DEFAULT_SCHEDULE: char = 'A';

pub trait CommandAble: Display {
    fn command(&self, id: u8) -> String<COMMAND_STRING_SIZE> {
        let mut payload = String::<{ COMMAND_STRING_SIZE - CHECKSUM_STRING_SIZE }>::new();
        write!(payload, "{}", self).unwrap();
        let checksum = checksum(&payload);
        let mut buffer = String::<COMMAND_STRING_SIZE>::new();
        write!(&mut buffer, "<ID{:02X}>{}{:02X}<E>", id, payload, checksum).unwrap();
        buffer
    }
}

pub fn set_id(id: u8) -> String<COMMAND_STRING_SIZE> {
    let mut buffer = String::<COMMAND_STRING_SIZE>::new();
    write!(&mut buffer, "<ID><{:02X}><E>", id).unwrap();
    buffer
}

fn checksum(payload: &str) -> u8 {
    let mut check: u8 = 0;
    for character in payload.as_bytes() {
        check ^= character;
    }
    check
}
