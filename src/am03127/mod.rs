mod page_content;
use core::fmt::Write;

use heapless::String;

pub fn set_id(id: u8) -> String<32> {
    let mut buffer = String::<32>::new();
    write!(&mut buffer, "<ID><{:02X}><E>", id).unwrap();
    buffer
}

fn wrap_command(id: u8, payload: &str) -> String<32> {
    let checksum = checksum(payload);
    let mut buffer = String::<32>::new();
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
