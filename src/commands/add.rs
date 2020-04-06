use std::path::PathBuf;

use crate::client::client::{Client};

use clap::ArgMatches;

pub fn command(add_cmd: &ArgMatches) {
    match add_cmd.value_of("FILE") {
        Some(torrent_file) => {
            let mut client = Client::new();
            let path = PathBuf::new().with_file_name(torrent_file);
            match client.add_torrent(&path) {
                Ok(_) => println!("Torrent added!"),
                Err(error) => println!("{}", error),
            };
        },
        None => println!("{}", add_cmd.usage().to_string()),
    };
}
