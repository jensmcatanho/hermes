use hermes::commands::add;

use clap::{App, AppSettings, load_yaml};

fn main() {
    let yaml = load_yaml!("hermes.yaml");
    let app = App::from_yaml(yaml).setting(AppSettings::ArgRequiredElseHelp);

    match app.get_matches_safe() {
        Ok(matches) => match matches.subcommand() {
            ("add", Some(torrent_file)) => add::command(torrent_file),
            _ => println!("aaa {}", matches.usage().to_string()),
        },
        Err(_) => {},
    }
}
