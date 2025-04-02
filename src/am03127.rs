use heapless::String;

const DEFAULT_ID: u8 = 1;

pub fn set_id(id: u8) -> String<32> {
    format!("<ID><{:02X}><E>", id)
}

fn wrap_command(id: u8, payload: &str) -> String<32> {
    let checksum = checksum(payload);
    format!("<ID{:02X}>{}{:02X}<E>", id, payload, checksum)
}

fn checksum(payload: &str) -> u8 {
    let mut check: u8 = 0;
    for character in payload.as_bytes() {
        check ^= character;
    }
    check
}
