use std::error;
use std::fmt;
use std::fs;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::string::String;
use std::vec::Vec;

use crate::internal::bencoding::{Decoder, BEncodedType};

#[derive(Debug, Clone)]
pub struct NewTorrentFromFileError;

impl fmt::Display for NewTorrentFromFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error initializing torrent from file")
    }
}

impl error::Error for NewTorrentFromFileError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

#[derive(Debug, Clone)]
struct MissingRequiredFieldError {
    field: String,
}

impl fmt::Display for MissingRequiredFieldError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Missing required field")
    }
}

impl error::Error for MissingRequiredFieldError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

const REQUIRED_FIELDS: [&'static str; 8] = [
    "info",
    "announce",
    "piece length",
    "pieces",
    "name",
    "length",
    "files",
    "path",
];

pub struct Torrent {
    pub comment: String,
    pub created_by: String,
    pub encoding: String,
    pub trackers: Vec<String>,
    pub is_private: bool,
    pub piece_length: i32,
    pub pieces: Vec<u8>,
    pub files: Vec<File>,
}

impl Default for Torrent {
    fn default() -> Torrent {
        Torrent{
            comment: String::new(),
            created_by: String::new(),
            encoding: String::new(),
            trackers: Vec::new(),
            is_private: false,
            piece_length: 0,
            pieces: Vec::new(),
            files: Vec::new(),
        }
    }
}

pub struct File {
    pub path: PathBuf,
    pub size: i64,
    pub offset: i64,
}

impl Torrent {
    pub fn new(path: &Path) -> Result<Torrent, NewTorrentFromFileError> {
        let encoded_metainfo = fs::read(&path).expect("Something went wrong reading the file");
        let decoder = Decoder::new(encoded_metainfo);

        match decoder.decode() {
            Ok(metainfo) => match Torrent::bencoding_to_torrent(*metainfo) {
                Ok(torrent) => Ok(torrent),
                Err(error) => {
                    println!("{}", error);
                    Err(NewTorrentFromFileError)
                },
            },
            Err(_) => Err(NewTorrentFromFileError)
        }
    }
    
    fn bencoding_to_torrent(data: HashMap<String, Box<dyn BEncodedType>>) -> Result<Torrent, MissingRequiredFieldError> {
        let mut torrent: Torrent = Torrent::default();
        
        let announce_key = "announce".to_string();
        let announce_value = Torrent::bencoded_to_string(&data, announce_key)?;
        torrent.trackers.push(announce_value);

        let created_by_key = "created by".to_string();
        let created_by_value = Torrent::bencoded_to_string(&data, created_by_key)?;
        torrent.created_by = created_by_value;
        
        let comment_key = "comment".to_string();
        let comment_value = Torrent::bencoded_to_string(&data, comment_key)?;
        torrent.comment = comment_value;
        
        let encoding_key = "encoding".to_string();
        let encoding_value = Torrent::bencoded_to_string(&data, encoding_key)?;
        torrent.encoding = encoding_value;
        
        let info_key = "info".to_string();
        let info = match data[&info_key].as_any().downcast_ref::<HashMap<String, Box<dyn BEncodedType>>>() {
            Some(dictionary_from_bencoded) => dictionary_from_bencoded,
            None => return Err(MissingRequiredFieldError{ field: info_key })
        };

        let piece_length_key = "piece length".to_string();
        let piece_length_value = Torrent::bencoded_to_i64(&info, piece_length_key)?;
        torrent.piece_length = piece_length_value as i32;

        let pieces_key = "pieces".to_string();
        let pieces_value = Torrent::bencoded_to_bytes(&info, pieces_key)?;
        torrent.pieces = pieces_value;

        let private_key = "private".to_string();
        torrent.is_private = Torrent::bencoded_to_bool(&info, private_key);
        
        let files_key = "files".to_string();

        match info[&files_key].as_any().downcast_ref::<Vec<Box<dyn BEncodedType>>>() {
            Some(files) => {
                let mut accumulated: i64 = 0;

                for file in files {
                    let file_info = match file.as_any().downcast_ref::<HashMap<String, Box<dyn BEncodedType>>>() {
                        Some(dictionary_from_bencoded) => dictionary_from_bencoded,
                        None => continue,
                    };

                    let path_key = "path".to_string();
                    let path_value = Torrent::bencoded_to_string(&file_info, path_key)?;

                    let size_key = "length".to_string();
                    let size_value = Torrent::bencoded_to_i64(&file_info, size_key)?;

                    torrent.files.push(File{
                        path: PathBuf::from(path_value),
                        size: size_value,
                        offset: accumulated,
                    });

                    accumulated += size_value;
                }
            },
            None => {
                let name_key = "name".to_string();
                let name_value = Torrent::bencoded_to_string(&info, name_key)?;
                
                let length_key = "length".to_string();
                let length_value = Torrent::bencoded_to_i64(&info, length_key)?;
                
                torrent.files.push(File{
                    path: PathBuf::from(name_value),
                    size: length_value,
                    offset: 0,
                });
            },
        };

        Ok(torrent)
    }

    fn bencoded_to_string(data: &HashMap<String, Box<dyn BEncodedType>>, field: String) -> Result<String, MissingRequiredFieldError> {
        let empty: String = String::new();

        if !data.contains_key(&field) && Torrent::is_required_field(&field) {
            return Err(MissingRequiredFieldError{ field: field })
        }

        match data[&field].as_any().downcast_ref::<Vec<u8>>() {
            Some(string_from_bencoded) => Ok(std::str::from_utf8(string_from_bencoded).unwrap().to_string()),
            None if Torrent::is_required_field(&field) => Err(MissingRequiredFieldError{ field: field }),
            None => Ok(empty),
        }
    }

    fn bencoded_to_bytes(data: &HashMap<String, Box<dyn BEncodedType>>, field: String) -> Result<Vec<u8>, MissingRequiredFieldError> {
        let empty: Vec<u8> = Vec::new();

        if !data.contains_key(&field) && Torrent::is_required_field(&field) {
            return Err(MissingRequiredFieldError{ field: field })
        }
        
        match data[&field].as_any().downcast_ref::<Vec<u8>>() {
            Some(bytes_from_bencoded) => Ok(bytes_from_bencoded.to_vec()),
            None if Torrent::is_required_field(&field) => Err(MissingRequiredFieldError{ field: field }),
            None => Ok(empty),
        }
    }

    fn bencoded_to_bool(data: &HashMap<String, Box<dyn BEncodedType>>, field: String) -> bool {
        match Torrent::bencoded_to_i64(data, field) {
            Ok(number) => number == !0,
            Err(_) => false,
        }
    }

    fn bencoded_to_i64(data: &HashMap<String, Box<dyn BEncodedType>>, field: String) -> Result<i64, MissingRequiredFieldError> {
        if !data.contains_key(&field) && Torrent::is_required_field(&field) {
            return Err(MissingRequiredFieldError{ field: field })
        }

        match data[&field].as_any().downcast_ref::<i64>() {
            Some(i64_from_bencoded) => Ok(*i64_from_bencoded),
            None if Torrent::is_required_field(&field) => Err(MissingRequiredFieldError{ field: field }),
            None => Ok(0),
        }
    }

    fn is_required_field(field: &String) -> bool {
        match REQUIRED_FIELDS.iter().find(|&s| *s == *field) {
            Some(_) => true,
            _ => false,
        }
    }
}