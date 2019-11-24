#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate websocket;
#[macro_use]
extern crate rust_embed;

use std::thread;
use websocket::sync::Server;
use websocket::OwnedMessage;

#[derive(RustEmbed)]
#[folder="views"]
struct Views;

#[get("/")]
fn index() -> String {
    let view = Views::get("index.html.hbs").unwrap();
    String::from_utf8_lossy(view.as_ref()).into_owned()
}

fn websocket() {
    let server = Server::bind("127.0.0.1:8001").unwrap();

    for request in server.filter_map(Result::ok) {
        // Spawn a new thread for each connection.
        thread::spawn(|| {
            if !request.protocols().contains(&"rust-websocket".to_string()) {
                request.reject().unwrap();
                return;
            }

            let mut client = request.use_protocol("rust-websocket").accept().unwrap();

            let ip = client.peer_addr().unwrap();

            println!("Connection from {}", ip);

            let message = OwnedMessage::Text("Hello".to_string());
            client.send_message(&message).unwrap();

            let (mut receiver, mut sender) = client.split().unwrap();

            for message in receiver.incoming_messages() {
                let message = message.unwrap();

                match message {
                    OwnedMessage::Close(_) => {
                        let message = OwnedMessage::Close(None);
                        sender.send_message(&message).unwrap();
                        println!("Client {} disconnected", ip);
                        return;
                    }
                    OwnedMessage::Ping(ping) => {
                        let message = OwnedMessage::Pong(ping);
                        sender.send_message(&message).unwrap();
                    }
                    _ => sender.send_message(&message).unwrap(),
                }
            }
        });
    }

}

fn main() {

    thread::spawn(|| {
        websocket()
    });

    rocket::ignite()
        .mount("/", routes![index]).launch();
}
