use crossbeam::channel::Receiver;
use serde_json;
use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::server::accept;

use crate::kafka::KittyMessage;

pub fn serve(rx: Receiver<KittyMessage>) {
    let server = TcpListener::bind("localhost:8001").unwrap();
    for stream in server.incoming() {
        let channel = rx.clone();
        spawn (move || {
            let mut websocket = accept(stream.unwrap()).unwrap();
            loop {
                let msg = channel.recv().unwrap();
                let buffer = serde_json::to_string(&msg).unwrap();
                websocket.write_message(tungstenite::Message::Text(buffer));
            }
        });
    }
}

