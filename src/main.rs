
use std::path::PathBuf;

use hermes::internal::client::Client;

fn main() {
    println!("hermes - BitTorrent client");
    let mut client = Client::new();
    let path = PathBuf::new().with_file_name("test.torrent");
    
    client.add_torrent(&path);
}
