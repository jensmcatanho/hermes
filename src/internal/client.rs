use std::path::Path;
use std::vec::Vec;

use crate::internal::torrent;

pub struct Client {
    torrents: Vec<torrent::Torrent>,
}

impl Client {
    pub fn new() -> Client {
        Client{
            torrents: Vec::new()
        }
    }

    pub fn add_torrent(&mut self, torrent_file: &Path) -> Result<(), torrent::NewTorrentFromFileError> {
        match torrent::Torrent::new(torrent_file) {
            Ok(torrent) => Ok(self.torrents.push(torrent)),
            Err(_) => Err(torrent::NewTorrentFromFileError),
        }
    }
}