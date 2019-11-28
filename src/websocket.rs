use crossbeam::channel::Receiver;
use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::server::accept;

pub fn serve(rx: Receiver<String>) {
    let server = TcpListener::bind("localhost:8001").unwrap();
    for stream in server.incoming() {
        let channel = rx.clone();
        spawn (move || {
            let mut websocket = accept(stream.unwrap()).unwrap();
            loop {
                let msg = channel.recv().unwrap();
                websocket.write_message(tungstenite::Message::Text(msg)).unwrap();
            }
        });
    }
}

