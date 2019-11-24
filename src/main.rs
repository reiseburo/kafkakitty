#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rust_embed;

use rocket_contrib::templates::Template;


#[derive(RustEmbed)]
#[folder="views"]
struct Views;

#[get("/")]
fn index() -> String {
    let view = Views::get("index.html.hbs").unwrap();
    String::from_utf8_lossy(view.as_ref()).into_owned()
}

fn main() {
    rocket::ignite()
        .attach(Template::fairing())
        .mount("/", routes![index]).launch();
}
