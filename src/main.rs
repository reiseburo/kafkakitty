#![feature(proc_macro_hygiene, decl_macro)]

extern crate crossbeam;
extern crate rdkafka;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rust_embed;
extern crate tungstenite;

use crossbeam::channel::{Sender, Receiver};

use futures::stream::Stream;

use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext};
use rdkafka::Message;
use rdkafka::message::Headers;

use rocket::http::{ContentType, Status};
use rocket::response;

use std::ffi::OsStr;
use std::io::Cursor;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::thread::spawn;
use std::time::Duration;

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
    /// A WebSocket echo server
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
    let (sender, receiver) = crossbeam::channel::unbounded::<String>();
    thread::spawn(move || {
        websocket(receiver);
    });

    thread::spawn(move || {
        consume_and_print("localhost:9092", "kafkakitty", &["test"], sender);
    });

    rocket::ignite()
        .mount("/", routes![index, assets]).launch();
}

fn consume_and_print(brokers: &str, group_id: &str, topics: &[&str], sender: Sender<String>) {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        //.set("statistics.interval.ms", "30000")
        //.set("auto.offset.reset", "smallest")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()
        .expect("Consumer creation failed");

    consumer.subscribe(&topics.to_vec())
        .expect("Can't subscribe to specified topics");

    // consumer.start() returns a stream. The stream can be used ot chain together expensive steps,
    // such as complex computations on a thread pool or asynchronous IO.
    let message_stream = consumer.start();

    for message in message_stream.wait() {
        match message {
            Err(_) => println!("Error while reading from stream."),
            Ok(Err(e)) => println!("Kafka error: {}", e),
            Ok(Ok(m)) => {
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        println!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };
                println!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                      m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());
                sender.send(payload.to_string()).unwrap();

                if let Some(headers) = m.headers() {
                    for i in 0..headers.count() {
                        let header = headers.get(i).unwrap();
                        println!("  Header {:#?}: {:?}", header.0, header.1);
                    }
                }
                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
        };
    }
}
