mod page_content;
use core::fmt::Write;

use heapless::String;

// Constants for string sizes
pub const SMALL_STRING_SIZE: usize = 32;
pub const MEDIUM_STRING_SIZE: usize = 64;
pub const LARGE_STRING_SIZE: usize = 128;

pub fn set_id(id: u8) -> String<SMALL_STRING_SIZE> {
    let mut buffer = String::<SMALL_STRING_SIZE>::new();
    write!(&mut buffer, "<ID><{:02X}><E>", id).unwrap();
    buffer
}

fn wrap_command(id: u8, payload: &str) -> String<SMALL_STRING_SIZE> {
    let checksum = checksum(payload);
    let mut buffer = String::<SMALL_STRING_SIZE>::new();
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
