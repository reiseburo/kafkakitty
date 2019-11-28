use crossbeam::channel::Receiver;
use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::server::accept;

pub fn serve(rx: Receiver<String>) {
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

