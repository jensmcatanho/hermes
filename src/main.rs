
use std::path::PathBuf;

use hermes::internal::torrent::Torrent;

fn main() {
    println!("hermes - BitTorrent client");

    let path = PathBuf::new().with_file_name("test.torrent");
    match Torrent::new(&path) {
        Ok(torrent) => println!("{}", torrent.piece_length),
        Err(error) => println!("{}", error),
    };

}
