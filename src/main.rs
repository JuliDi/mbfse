#![feature(plugin, register_attr, proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::{uri, Data, response::{status::Custom, content::Html, Redirect}, http::{Status, ContentType}};
use multipart::server::{Multipart, ReadEntry, MultipartField, save::SaveResult::*, save::SavedData::File};

use std::{path::{PathBuf}, env::var};
use nanoid::nanoid;
use lazy_static::lazy_static;

const HEADER: &str = r##"<div style="font-family:monospace; text-align: center; zoom: 1.5; padding-left: 3vw; padding-top:20vh;">"##;

lazy_static! {
    static ref STORAGE_PATH: String = var("STORAGE_PATH").unwrap_or(String::new());
    static ref BASE_URL: String = var("BASE_URL").unwrap_or(String::new());
}

/// ### Program entry function
/// Load variables from ENV and print an error if they are missing
/// Then: ignite the rocket!
fn main() {
    let mut error = false;
    if BASE_URL.is_empty() {
        eprintln!("BASE_URL not set");
        error = true;
    }
    if STORAGE_PATH.is_empty() {
        eprintln!("STORAGE_PATH not set");
        error = true;
    }
    if error {
        eprintln!("Errors occured.");
        std::process::exit(-1)
    }
    rocket::ignite().mount("/", routes![index,multipart_upload,success]).launch();
}

///
/// ### Show homepage
/// Displays a short text and provides two buttons: one for choosing a file and one to upload it
#[get("/")]
fn index() -> Result<Html<String>, Status> {
    let content = format!(r##"{}
<form action="/upload" method="post" enctype="multipart/form-data">
   <h1>Select file to upload:</h1> <br>
   <input type="file" name="fileToUpload" id="fileToUpload" required> <br><br>
   <input type="submit" value="Upload File" name="submit">
</form>
</div>
"##, HEADER);
    Ok(Html(content))
}

/// ### Upload file function
/// This page is loaded when the file selection form is submitted.
/// The function's signature requires the request to have a `Content-Type` that can be used
/// to verify that the form data is a `multipart/form-data` and is bounded (which should help
/// mitigating attacks).
/// If those checks have passed, it will call [`process_upload`] to actually copy the file to
/// file server and redirect the user to the [`success`] page.
#[post("/upload", data = "<data>")]
fn multipart_upload(cont_type: &ContentType, data: Data) -> Result<Redirect, Custom<String>> {
    // this and the next check can be implemented as a request guard but it seems like just
    // more boilerplate than necessary
    if !cont_type.is_form_data() {
        return Err(Custom(
            Status::BadRequest,
            "Content-Type not multipart/form-data".into(),
        ));
    }

    let (_, boundary) = cont_type.params().find(|&(k, _)| k == "boundary").ok_or(
        Custom(Status::BadRequest, "`Content-Type: multipart/form-data` boundary param not provided".into())
    )?;

    match process_upload(boundary, data) {
        Ok(file) => {
            let filename = file.file_name().ok_or(Custom(Status::InternalServerError, "no valid filename for stored file found!".into()))?.to_str().ok_or(Custom(Status::InternalServerError, "filename not valid UTF-8".into()))?;
            // println!("file = {}", &filename); // debug only
            Ok(Redirect::to(uri!(success: filename)))
        }
        Err(err) => Err(Custom(Status::InternalServerError, err))
    }
}

/// Extract the file from the [`Data`] object and save it to the file server
/// ## Arguments
/// - `boundary`: boundary string as documented in IETF RFC 1341
/// - `data`: [`Data`] object originating from the POST request
fn process_upload(boundary: &str, data: Data) -> Result<PathBuf, String> {
    // create a new `multipart` fro further processing
    let mut multipart = Multipart::with_body(data.open(), boundary);
    // get all entries and find the field "fileToUpload"

    let mut res = Err("no fileToUpload field found".to_string());

    multipart.foreach_entry(|mut e| {
        if &*e.headers.name == "fileToUpload" {
            res = save_file(&mut e);
        }
    }).map_err(|e| { e.to_string() })?;
    res
}

/// Save a [`MultipartField`] to a file on the specified [`STORAGE_PATH`] and return the filename
/// as a [`String`] or an Error, if the upload failed.
/// 1. a random filename is created using [`nanoid!`].
/// 2. the extension, if existent, from the original filename is appended to the random filename
/// 3. The file is saved to `<STORAGE_PATH>/<filename>.<extension>
fn save_file<M: ReadEntry>(e: &mut MultipartField<M>) -> Result<PathBuf, String> {
    // Create a random base name for the file (avoid any malicious code injection by the filename)
    let mut filename = PathBuf::from(nanoid!(20));

    // If there is a filename, get the extension and set it for the previously created `filename`
    if let Some(ext) = &e.headers.filename {
        filename.set_extension(PathBuf::from(ext).extension().unwrap().to_str().unwrap());
    }

    // Create a full path from the STORAGE_PATH and the file base name
    let filepath = PathBuf::from(&*STORAGE_PATH).join(&filename);

    // save file from the form to the full filepath
    // IMPORTANT: Set memory_threshold to `0` to avoid buffering (i.e. write to a file regardless of the size)
    let saved_file = e.data.save().memory_threshold(0).ignore_text().with_path(&filepath);
    match saved_file {
        Full(success) => {
            if let File(path, _) = success { Ok(path) } else { Err("No file found".into()) }
        }
        Partial(_, _) => { Err("found partial".into()) }
        Error(e) => Err(e.to_string())
    }
}

/// Display a success page that includes a link to the saved file for sharing
#[get("/success/<file>")]
fn success(file: String) -> Html<String> {
    Html(format!(r#"{}
            <h1>File link:</h1>
            <br>
            <a href="{}/{}">{}/{}</a>
            <br><br>
            <a href="/">Upload next file</a>
        </div>
        "#, HEADER, &*BASE_URL, &file, &*BASE_URL, &file))
}
