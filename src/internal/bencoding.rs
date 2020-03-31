use std::str;

const TOKEN_START: u8 = 100;
const TOKEN_END: u8 = 101;
const TOKEN_LIST: u8 = 108;
const TOKEN_NUMBER: u8 = 105;
const TOKEN_DELIMITER: u8 = 58;

pub struct Decoder ();

impl Decoder {
    pub fn decode(file: &'static str) -> &'static str {
        file
    }
}