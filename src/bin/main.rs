extern crate clap;

#[macro_use]
extern crate log;

use std::path::Path;

use clap::{App, Arg};
use env_logger::{Builder, Env};
use tokio::runtime;

use monkeylib::Service;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Builder::from_env(Env::default()).init();

    let matches = App::new("Monkey")
        .version("1.0")
        .author("Rohit Narurkar <rohit.narurkar@protonmail.com>")
        .about("Monkey is a command-line P2P toy blockchain")
        .arg(
            Arg::with_name("db")
                .help("directory name for database")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("peer")
                .help("multiaddr of a peer")
                .takes_value(true),
        )
        .get_matches();

    let to_dial = match matches.value_of("peer") {
        Some(peer) => match peer.parse() {
            Ok(to_dial) => Some(to_dial),
            Err(err) => {
                error!("Failed to parse peer to dial: {:?}", err);
                None
            }
        },
        _ => None,
    };

    let db_name = matches.value_of("db").unwrap();

    let rt = runtime::Builder::new()
        .threaded_scheduler()
        .core_threads(4)
        .build()
        .unwrap();

    let rt_handle = rt.handle();

    let path = Path::new(".data").join(".blockchain").join(db_name);
    let mut service = Service::new(&rt_handle, &path).ok().unwrap();
    service.start(&rt_handle, to_dial)?;

    Ok(())
}
