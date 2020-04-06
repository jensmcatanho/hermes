use std::error;
use std::fmt;

use std::any::Any;
use std::boxed::Box;
use std::collections::HashMap;
use std::string::String;
use std::vec::Vec;

const TOKEN_START: u8 = b'd';
const TOKEN_END: u8 = b'e';
const TOKEN_LIST: u8 = b'l';
const TOKEN_NUMBER: u8 = b'i';
const TOKEN_DELIMITER: u8 = b':';

#[derive(Debug, Clone)]
pub struct DecodeError;

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error decoding .torrent file")
    }
}

impl error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

struct BEncodedData {
    index: usize,
    data: std::vec::Vec<u8>,
}

impl BEncodedData {
    fn new(data: std::vec::Vec<u8>) -> BEncodedData {
        BEncodedData {
            index: 0,
            data: data,
        }
    }

    fn current(&mut self) -> Option<u8> {
        if self.index < self.data.len() {
            Some(self.data[self.index])
        } else {
            None
        }
    }
}

impl Iterator for BEncodedData {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;

        if self.index < self.data.len() {
            Some(self.data[self.index])
        } else {
            None
        }
    }
}

pub trait BEncodedType {
    fn as_any(&self) -> &dyn Any;
}

impl BEncodedType for i64 {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl BEncodedType for Vec<u8> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl BEncodedType for Vec<Box<dyn BEncodedType>> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl BEncodedType for HashMap<String, Box<dyn BEncodedType>> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}


pub struct Decoder {
    encoded_data: BEncodedData,
}

impl Decoder {
    pub fn new(encoded_data: Vec<u8>) -> Decoder {
        Decoder{
            encoded_data: BEncodedData::new(encoded_data),
        }
    }

    pub fn decode(mut self) -> Result<Box<HashMap<String, Box<dyn BEncodedType>>>, DecodeError> {
        match self.decode_dictionary() {
            Ok(dictionary) => Ok(dictionary),
            Err(_) => Err(DecodeError),
        }
    }

    fn decode_dictionary(&mut self) -> Result<Box<HashMap<String, Box<dyn BEncodedType>>>, DecodeError> {
        let mut decoded_dictionary: HashMap<String, Box<dyn BEncodedType>> = HashMap::new();
        self.encoded_data.next().unwrap();

        loop {
            let key = match self.encoded_data.current() {
                Some(token) if token == TOKEN_END => break,
                Some(_) => match self.decode_string() {
                    Ok(key) => match String::from_utf8(*key) {
                        Ok(key_from_utf8) => key_from_utf8,
                        Err(_) => return Err(DecodeError),
                    },
                    Err(_) => return Err(DecodeError),
                },
                None => return Err(DecodeError)
            };

            match self.encoded_data.current() {
                Some(token) if token == TOKEN_END => break,
                Some(token) if token == TOKEN_START => {
                    match self.decode_dictionary() {
                        Ok(dictionary) => decoded_dictionary.insert(key, dictionary),
                        Err(_) => return Err(DecodeError),
                    };
                },
                Some(token) if token == TOKEN_NUMBER => {
                    match self.decode_number() {
                        Ok(number) => decoded_dictionary.insert(key, number),
                        Err(_) => return Err(DecodeError),
                    };
                },
                Some(token) if token == TOKEN_LIST => {
                    match self.decode_list() {
                        Ok(list) => decoded_dictionary.insert(key, list),
                        Err(_) => return Err(DecodeError),
                    };
                },
                Some(_) => {
                    match self.decode_string() {
                        Ok(string) => decoded_dictionary.insert(key, string),
                        Err(_) => return Err(DecodeError),
                    };
                },
                None => return Err(DecodeError),
            }
        }

        self.encoded_data.next();
        Ok(Box::new(decoded_dictionary))
    }

    fn decode_string(&mut self) -> Result<Box<Vec<u8>>, DecodeError> {
        let mut length_bytes: std::vec::Vec<u8> = Vec::new();
        
        loop {
            match self.encoded_data.current() {
                Some(current_token) if current_token != TOKEN_DELIMITER => length_bytes.push(current_token),
                Some(_) => break,
                None => return Err(DecodeError),
            };

            self.encoded_data.next();
        }

        let length_string = String::from_utf8(length_bytes).unwrap();
        let length = length_string.parse::<usize>().unwrap();

        let mut bytes: std::vec::Vec<u8>;
        bytes = Vec::with_capacity(length);
        for _ in 0..length {
            match self.encoded_data.next() {
                Some(token) => bytes.push(token),
                None => return Err(DecodeError),
            }
        }

        self.encoded_data.next();
        Ok(Box::new(bytes))
    }

    fn decode_number(&mut self) -> Result<Box<i64>, DecodeError> {
        let mut number_bytes: std::vec::Vec<u8> = Vec::new();

        match self.encoded_data.next() {
            Some(_) => (),
            None => return Err(DecodeError),
        };

        loop {
            match self.encoded_data.current() {
                Some(current_token) if current_token != TOKEN_END => number_bytes.push(current_token),
                Some(_) => break,
                None => return Err(DecodeError),
            };

            self.encoded_data.next();
        }

        let number_string = match String::from_utf8(number_bytes) {
            Ok(number) => number,
            Err(_) => return Err(DecodeError)
        };

        self.encoded_data.next();
    
        match number_string.parse::<i64>() {
            Ok(number) => Ok(Box::new(number)),
            Err(_) => Err(DecodeError),
        }
    }

    fn decode_list(&mut self) -> Result<Box<std::vec::Vec<Box<dyn BEncodedType>>>, DecodeError> {
        let mut decoded_list: std::vec::Vec<Box<dyn BEncodedType>> = Vec::new();

        self.encoded_data.next();

        loop {
            match self.encoded_data.current() {
                Some(token) if token == TOKEN_END => break,
                Some(token) if token == TOKEN_START => {
                    match self.decode_dictionary() {
                        Ok(dictionary) => decoded_list.push(dictionary),
                        Err(_) => return Err(DecodeError),
                    };
                },
                Some(token) if token == TOKEN_NUMBER => {
                    match self.decode_number() {
                        Ok(number) => decoded_list.push(number),
                        Err(_) => return Err(DecodeError),
                    };
                },
                Some(token) if token == TOKEN_LIST => {
                    match self.decode_list() {
                        Ok(list) => decoded_list.push(list),
                        Err(_) => return Err(DecodeError),
                    };
                },
                Some(_) => {
                    match self.decode_string() {
                        Ok(string) => decoded_list.push(string),
                        Err(_) => return Err(DecodeError),
                    };
                },
                None => return Err(DecodeError),
            }
        }

        self.encoded_data.next();
        Ok(Box::new(decoded_list))
    }
}