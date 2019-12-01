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

use log::warn;

use std::thread;
use std::time::Duration;

fn main() {
    let matches = App::new("Kafkakitty 😿")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or(""))
        .about("A cute little Kafka consumer")
        .arg(
            Arg::with_name("brokers")
                .short("b")
                .long("brokers")
                .help("Broker list in kafka format")
                .takes_value(true)
                .default_value("localhost:9092"),
        )
        .arg(
            Arg::with_name("group-id")
                .short("G")
                .long("group-id")
                .help("Consumer group ID for Kafka")
                .takes_value(true)
                .default_value("kafkakitty"),
        )
        .arg(
            Arg::with_name("topics")
                .short("t")
                .long("topics")
                .help("List of topics to follow")
                .takes_value(true)
                .multiple(true)
                .required(true),
        )
        .arg(
            Arg::with_name("no-open")
                .short("n")
                .long("no-open")
                .help("Disable opening a browser window automatically")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("settings")
                .short("X")
                .help("Set a librdkafka configuration property")
                .takes_value(true)
                .multiple(true),
        )
        .get_matches();

    let should_open = !matches.is_present("no-open");

    let mut settings: Vec<(String, String)> = vec![(String::from("group.id"), String::from("fo"))];

    // Default settings for librdkafka
    for (key, value) in [
        ("enable.partition.eof", "false"),
        ("session.timeout.ms", "6000"),
        ("enable.auto.commit", "true"),
    ]
    .iter()
    {
        settings.push((key.to_string(), value.to_string()));
    }

    settings.push((
        String::from("group.id"),
        String::from(matches.value_of("group-id").unwrap()),
    ));

    settings.push((
        String::from("bootstrap.servers"),
        String::from(matches.value_of("brokers").unwrap()),
    ));

    /*
     * Process the arbitrary settings passed through via the -X arguments
     *
     * These must be processed last since they should overwrite defaults if they exist
     */
    if matches.is_present("settings") {
        let x_args = matches
            .values_of("settings")
            .unwrap()
            .collect::<Vec<&str>>();
        for arg in x_args {
            let parts = arg.split("=").collect::<Vec<&str>>();

            if parts.len() == 2 {
                settings.push((parts[0].to_string(), parts[1].to_string()));
            } else {
                warn!("Could not understand the setting `{}`, skipping", arg);
            }
        }
    }

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

        kafka::consume(sender, settings, &topics);
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
        .mount("/", routes![routes::index, routes::assets])
        .launch();
}
