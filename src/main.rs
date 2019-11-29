#![feature(proc_macro_hygiene, decl_macro)]

extern crate crossbeam;
extern crate open;
extern crate rdkafka;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rust_embed;
extern crate tungstenite;

mod kafka;
mod routes;
mod websocket;

use clap::{App, Arg};

use log::info;

use std::thread;
use std::time::Duration;

fn main() {
    let matches = App::new("Kafkakitty 😿")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or(""))
        .about("A cute little Kafka consumer")
        .arg(Arg::with_name("brokers")
             .short("b")
             .long("brokers")
             .help("Broker list in kafka format")
             .takes_value(true)
             .default_value("localhost:9092"))
        .arg(Arg::with_name("group-id")
             .short("G")
             .long("group-id")
             .help("Consumer group ID for Kafka")
             .takes_value(true)
             .default_value("kafkakitty"))
        .arg(Arg::with_name("topics")
             .short("t")
             .long("topics")
             .help("List of topics to follow")
             .takes_value(true)
             .multiple(true)
             .required(true))
        .arg(Arg::with_name("no-open")
             .short("n")
             .long("no-open")
             .help("Disable opening a browser window automatically")
             .takes_value(false))
        .get_matches();

    let should_open = ! matches.is_present("no-open");

    let (sender, receiver) = crossbeam::channel::unbounded::<kafka::KittyMessage>();

    /*
     * Kafkakitty requires at minimum three threads:
     *   - The websocket server thread
     *   - The Rocket server thread
     *   - The Kafka consumer thread
     *
     * There's communication between the websocket and consumer thread via channels
     * provided by the crossbeam crate.
     *
     * This allows multiple browser windows to receive the same messages
     */
    thread::spawn(move || {
        websocket::serve(receiver);
    });

    thread::spawn(move || {
        let topics = matches.values_of("topics").unwrap().collect::<Vec<&str>>();
        let brokers = matches.value_of("brokers").unwrap();
        let group_id = matches.value_of("group-id").unwrap();
        info!("Setting up consumer for {}", brokers);

        kafka::consume(sender, brokers, group_id, &topics);
    });

    if should_open {
        thread::spawn(move || {
            // Sleep for a second just to give the server a chance to bootstrap
            thread::sleep(Duration::from_millis(1000));
            open::that("http://localhost:8000/")
                .expect("Unable to open a browser, you'll have to do that yourself!");
        });
    }

    // Launch the web app
    rocket::ignite()
        .mount("/", routes![routes::index, routes::assets]).launch();
}
