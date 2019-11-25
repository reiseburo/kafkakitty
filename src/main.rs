#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate tungstenite;
#[macro_use]
extern crate rust_embed;

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

fn websocket() {
    /// A WebSocket echo server
    let server = TcpListener::bind("127.0.0.1:8001").unwrap();
    for stream in server.incoming() {
        println!("Connection..");
        spawn (move || {
            let mut websocket = accept(stream.unwrap()).unwrap();
            loop {
                let msg = websocket.read_message().unwrap();

                // We do not want to send back ping/pong messages.
                if msg.is_binary() || msg.is_text() {
                    websocket.write_message(msg).unwrap();
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
