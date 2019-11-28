#![feature(proc_macro_hygiene, decl_macro)]

extern crate crossbeam;
extern crate rdkafka;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rust_embed;
extern crate tungstenite;

use clap::{App, Arg};
use crossbeam::channel::{Sender, Receiver};
use futures::stream::Stream;

use log::{info, warn};

use rdkafka::config::ClientConfig;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer};
use rdkafka::Message;
use rdkafka::message::Headers;

use rocket::http::{ContentType, Status};
use rocket::response;

use std::ffi::OsStr;
use std::io::Cursor;
use std::net::TcpListener;
use std::path::PathBuf;
use std::thread;
use std::thread::spawn;

use tungstenite::server::accept;

#[derive(RustEmbed)]
#[folder="assets"]
struct Assets;

#[get("/")]
fn index<'r>() -> response::Result<'r> {
  Assets::get("index.html").map_or_else(
    || Err(Status::NotFound),
    |d| response::Response::build().header(ContentType::HTML).sized_body(Cursor::new(d)).ok(),
  )
}

#[get("/assets/<file..>")]
fn assets<'r>(file: PathBuf) -> response::Result<'r> {
  let filename = file.display().to_string();
  Assets::get(&filename).map_or_else(
    || Err(Status::NotFound),
    |d| {
      let ext = file
        .as_path()
        .extension()
        .and_then(OsStr::to_str)
        .ok_or(Status::new(400, "Could not get file extension"))?;
      let content_type = ContentType::from_extension(ext).ok_or(Status::new(400, "Could not get file content type"))?;
      response::Response::build().header(content_type).sized_body(Cursor::new(d)).ok()
    },
  )
}

fn websocket(rx: Receiver<String>) {
    // A WebSocket echo server
    let server = TcpListener::bind("127.0.0.1:8001").unwrap();
    for stream in server.incoming() {
        let channel = rx.clone();
        println!("Connection..");
        spawn (move || {
            let mut websocket = accept(stream.unwrap()).unwrap();
            loop {
                let msg = channel.recv().unwrap();
                websocket.write_message(tungstenite::Message::Text(msg)).unwrap();
            }
        });
    }
}

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

    let (sender, receiver) = crossbeam::channel::unbounded::<String>();

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
        websocket(receiver);
    });

    thread::spawn(move || {
        let topics = matches.values_of("topics").unwrap().collect::<Vec<&str>>();
        let brokers = matches.value_of("brokers").unwrap();
        let group_id = matches.value_of("group-id").unwrap();
        info!("Setting up consumer for {}", brokers);

        consume(sender, brokers, group_id, &topics);
    });

    // Launch the web app
    rocket::ignite()
        .mount("/", routes![index, assets]).launch();
}

/**
 * The consume function is solely responsible for consuming from the given Kafka
 * topics and sending the message over the channel via the `tx` Sender
 *
 */
fn consume(tx: Sender<String>,
           brokers: &str,
           group_id: &str,
           topics: &[&str]) {

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .create()
        .expect("Consumer creation failed");

    consumer.subscribe(&topics.to_vec())
        .expect("Can't subscribe to specified topics");

    for message in consumer.start().wait() {
        match message {
            Err(_) => warn!("Error while reading from stream."),
            Ok(Err(e)) => warn!("Kafka error: {}", e),
            Ok(Ok(m)) => {
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        println!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };
                info!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                      m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());
                tx.send(payload.to_string()).unwrap();

                if let Some(headers) = m.headers() {
                    for i in 0..headers.count() {
                        let header = headers.get(i).unwrap();
                        info!("  Header {:#?}: {:?}", header.0, header.1);
                    }
                }
                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
        };
    }
}
