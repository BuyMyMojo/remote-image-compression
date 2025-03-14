use std::{
    io::Read,
    os::unix::fs::MetadataExt,
    process::Command,
};

use actix_multipart::form::{MultipartForm, tempfile::TempFile};
use actix_web::{
    App, Either, Error, HttpResponse, HttpServer, Responder, get, post,
};
use anyhow::format_err;
use tempfile::NamedTempFile;
use tracing::{Level, event, instrument};
// use serde::Deserialize;

// #[derive(Debug, Deserialize)]
// struct Metadata {
//     name: String,
// }

type _HandledResult = Either<HttpResponse, anyhow::Result<(), Error>>;

#[derive(Debug, MultipartForm)]
struct UploadImageForm {
    #[multipart(limit = "100MB")]
    file: TempFile,
    // json: MpJson<Metadata>,
}

#[instrument]
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[instrument]
#[post("/echo-image")]
async fn echo_image(MultipartForm(form): MultipartForm<UploadImageForm>) -> impl Responder {
    match is_file_image(&form.file.content_type) {
        Some(x) => return x,
        None => {}
    }

    let mut reader = std::io::BufReader::new(form.file.file.as_file());
    let file_contents = &mut vec![];

    let _ = reader.read_to_end(file_contents);

    return HttpResponse::Ok()
        .content_type(form.file.content_type.unwrap())
        .body(file_contents.clone());
}

#[instrument]
#[post("/jpeg")]
async fn jpeg(MultipartForm(form): MultipartForm<UploadImageForm>) -> impl Responder {
    
    match is_file_image(&form.file.content_type) {
        Some(x) => return x,
        None => {}
    }

    let mut reader = std::io::BufReader::new(form.file.file.as_file());
    let input_file_contents = &mut vec![];
    let _ = reader.read_to_end(input_file_contents);
    drop(reader);

    // let img = ImageReader::new(Cursor::new(input_file_contents)).with_guessed_format().unwrap().decode().unwrap();
    let jpeg = tempfile::NamedTempFile::new().unwrap();

    let status = encode_jpeg(&form.file.file, &jpeg, Some(5)).await;
    if status.is_err() {
        event!(Level::ERROR, "jpeg compression failed:\n{:?}", status.unwrap());
        return HttpResponse::InternalServerError().body("Compression failed");
    }

    let mut reader = std::io::BufReader::new(&jpeg);
    let output_file_contents = &mut vec![];
    let _ = reader.read_to_end(output_file_contents);

    let size_percentage = jpeg.as_file().metadata().unwrap().size() * 100 / form.file.size as u64;

    event!(
        Level::INFO,
        "{} is now {}% of origional size as an {}!",
        &form.file.content_type.unwrap(),
        size_percentage,
        "image/jpeg"
    );

    return HttpResponse::Ok()
        .content_type(mime::IMAGE_JPEG)
        .append_header((
            "content-disposition",
            format!("filename=\"{}.jpeg\"", form.file.file_name.unwrap()),
        ))
        .body(output_file_contents.clone());
}

#[instrument]
async fn encode_jpeg(input: &NamedTempFile, output: &NamedTempFile, quality: Option<u8>) -> Result<(), anyhow::Error> {
    let jpeg_quality = match quality {
        Some(x) => {
            if x > 100 {
                return Err(format_err!("Invalid quality value"));
            } else {
                x
            }
        }
        None => 75,
    };

    let jpeg_command = format!(
        "magick {} ppm:- | cjpeg -outfile {} -quality {}",
        input.path().display(),
        output.path().display(),
        jpeg_quality
    );

    let optimise_command = format!(
        "jpegoptim {}",
        output.path().display()
    );

    let _output = Command::new("sh")
        .arg("-c")
        .arg(jpeg_command)
        .output()
        .expect("failed to execute process");

    // event!(Level::INFO, "{:?}", _output);
    
    let _output = Command::new("sh")
        .arg("-c")
        .arg(optimise_command)
        .output()
        .expect("failed to execute process");

    event!(Level::INFO, "{:?}", _output);

    Ok(())
}

/// Returns `None` if file content_type starts with 'image' otherwise returns with an error response to serve to client
#[instrument]
fn is_file_image(input_mime: &Option<mime::Mime>) -> Option<HttpResponse> {
    if input_mime.is_none() {
        return Some(HttpResponse::BadRequest().body("Bad file content type?"));
    }

    // This long ass line is just to get the first half of the content type. with 'image/jpeg' it would produce 'image' and then get cast it as a str for comparison
    if input_mime.clone().unwrap().type_().as_str() != "image" {
        return Some(HttpResponse::BadRequest().body("File uploaded in not a supported format!"));
    }

    return None;
}

#[actix_web::main]
#[instrument]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    HttpServer::new(|| App::new().service(hello).service(echo_image).service(jpeg))
        .bind(("127.0.0.1", 3970))?
        .run()
        .await
}
