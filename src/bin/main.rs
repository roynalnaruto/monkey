extern crate clap;

use std::path::Path;

use clap::{App, Arg};
use env_logger::{Builder, Env};

use monkeylib::Service;

fn main() {
    Builder::from_env(Env::default()).init();

    let matches = App::new("Monkey")
        .version("1.0")
        .author("Rohit Narurkar <rohit.narurkar@protonmail.com>")
        .about("Monkey is a command-line P2P toy blockchain")
        .arg(
            Arg::with_name("peer")
                .help("multiaddr of a peer in Monkey")
                .short("p")
                .takes_value(true),
        )
        .get_matches();

    let to_dial = match matches.value_of("peer") {
        Some(peer) => match peer.parse() {
            Ok(to_dial) => Some(to_dial),
            Err(err) => {
                println!("Failed to parse peer to dial: {:?}", err);
                None
            }
        },
        _ => None,
    };

    let path = Path::new(".data").join(".blockchain").join("chaindata");
    let mut service = Service::new(&path).ok().unwrap();
    service.start(to_dial);
}
