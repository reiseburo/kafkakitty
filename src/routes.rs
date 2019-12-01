/**
 * The routes module has all (both!) routes needed for Rocket
 *
 * It's VERY COMPLEX.
 */
use rocket::http::{ContentType, Status};
use rocket::response;
use std::ffi::OsStr;
use std::io::Cursor;
use std::path::PathBuf;

#[derive(RustEmbed)]
#[folder = "assets"]
struct Assets;

#[get("/")]
pub fn index<'r>() -> response::Result<'r> {
    Assets::get("index.html").map_or_else(
        || Err(Status::NotFound),
        |d| {
            response::Response::build()
                .header(ContentType::HTML)
                .sized_body(Cursor::new(d))
                .ok()
        },
    )
}

#[get("/assets/<file..>")]
pub fn assets<'r>(file: PathBuf) -> response::Result<'r> {
    let filename = file.display().to_string();
    Assets::get(&filename).map_or_else(
        || Err(Status::NotFound),
        |d| {
            let ext = file
                .as_path()
                .extension()
                .and_then(OsStr::to_str)
                .ok_or(Status::new(400, "Could not get file extension"))?;
            let content_type = ContentType::from_extension(ext)
                .ok_or(Status::new(400, "Could not get file content type"))?;
            response::Response::build()
                .header(content_type)
                .sized_body(Cursor::new(d))
                .ok()
        },
    )
}
