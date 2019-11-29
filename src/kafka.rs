extern crate serde;

use crossbeam::channel::Sender;
use futures::stream::Stream;
use log::{info, warn};

use rdkafka::config::ClientConfig;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer};
use rdkafka::Message;
use rdkafka::message::Headers;

use serde::{Serialize, Deserialize};

/**
 * A KittyMessage contains the necessary metadata for sending the message over
 * to the frontend
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct KittyMessage {
    offset: i64,
    partition: i32,
    payload: String,
    topic: String,
}

/**
 * The consume function is solely responsible for consuming from the given Kafka
 * topics and sending the message over the channel via the `tx` Sender
 *
 */
pub fn consume(tx: Sender<KittyMessage>,
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
                let km = KittyMessage {
                    offset: m.offset(),
                    partition: m.partition(),
                    payload: payload.to_string(),
                    topic: m.topic().to_string(),
                };
                tx.send(km).unwrap();

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
