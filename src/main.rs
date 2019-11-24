#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate websocket;
#[macro_use]
extern crate rust_embed;


use std::ffi::OsStr;
use std::io::Cursor;
use std::path::PathBuf;
use std::thread;
use rocket::http::{ContentType, Status};
use rocket::response;
use websocket::sync::Server;
use websocket::OwnedMessage;

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
        .mount("/", routes![index, assets]).launch();
}
