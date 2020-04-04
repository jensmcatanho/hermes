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

    pub fn decode(mut self) -> Box<HashMap<String, Box<dyn BEncodedType>>> {
        self.decode_dictionary()
    }

    fn decode_dictionary(&mut self) -> Box<HashMap<String, Box<dyn BEncodedType>>> {
        let mut decoded_dictionary: HashMap<String, Box<dyn BEncodedType>> = HashMap::new();
        self.encoded_data.next().unwrap();

        loop {
            let mut current_token = self.encoded_data.current().unwrap();
            if current_token == TOKEN_END { break; }

            let key = self.decode_string();
            let key_string = String::from_utf8(*key).unwrap();
            
            current_token = self.encoded_data.current().unwrap();

            if current_token == TOKEN_NUMBER {
                let number = self.decode_number();
                decoded_dictionary.insert(key_string, number);

            } else if current_token == TOKEN_LIST {
                let list = self.decode_list();
                decoded_dictionary.insert(key_string, list);

            } else if current_token == TOKEN_START {
                let dictionary = self.decode_dictionary();
                decoded_dictionary.insert(key_string, dictionary);

            } else {
                let string = self.decode_string();
                decoded_dictionary.insert(key_string, string);
            }
        }

        self.encoded_data.next();
        Box::new(decoded_dictionary)
    }

    fn decode_string(&mut self) -> Box<Vec<u8>> {
        let mut length_bytes: std::vec::Vec<u8> = Vec::new();

        while self.encoded_data.current().unwrap() != TOKEN_DELIMITER {
            let current_token = self.encoded_data.current().unwrap();
            length_bytes.push(current_token);

            self.encoded_data.next();
        }

        let length_string = String::from_utf8(length_bytes).unwrap();
        let length = length_string.parse::<usize>().unwrap();

        let mut bytes: std::vec::Vec<u8>;
        bytes = Vec::with_capacity(length);
        for _ in 0..length {
            let current_token = self.encoded_data.next().unwrap();
            bytes.push(current_token);
        }

        self.encoded_data.next();
        Box::new(bytes)
    }

    fn decode_number(&mut self) -> Box<i64> {
        let mut number_bytes: std::vec::Vec<u8> = Vec::new();
        
        self.encoded_data.next().unwrap();

        loop {
            let current_token = self.encoded_data.current().unwrap();
            if current_token == TOKEN_END { break; }

            number_bytes.push(current_token);
            self.encoded_data.next().unwrap();
        }

        let number_string = String::from_utf8(number_bytes).unwrap();
    
        self.encoded_data.next();
        Box::new(number_string.parse::<i64>().unwrap())
    }

    fn decode_list(&mut self) -> Box<std::vec::Vec<Box<dyn BEncodedType>>> {
        let mut decoded_list: std::vec::Vec<Box<dyn BEncodedType>> = Vec::new();

        self.encoded_data.next().unwrap();

        loop {
            let current_token = self.encoded_data.current().unwrap();
            if current_token == TOKEN_END { break; }

            if current_token == TOKEN_START {
                let dictionary = self.decode_dictionary();
                decoded_list.push(dictionary);

            } else if current_token == TOKEN_NUMBER {
                let number = self.decode_number();
                decoded_list.push(number);

            } else if current_token == TOKEN_LIST {
                let list = self.decode_list();
                decoded_list.push(list);

            } else {
                let string = self.decode_string();
                decoded_list.push(string);
            }
        }

        self.encoded_data.next();
        Box::new(decoded_list)
    }
}