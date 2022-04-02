#[macro_use] extern crate rocket;


use reqwest::ClientBuilder;
use std::time::Duration;
use std::fmt;
use std::io::Cursor;

use rocket::request::Request;
use rocket::response::{Responder, Response, Redirect};

#[derive(Debug)]
pub enum RequestError {
    Request(reqwest::Error),
    Io(std::io::Error),
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<reqwest::Error> for RequestError {
    fn from(error: reqwest::Error) -> Self {
        RequestError::Request(error)
    }
}

impl<'r> Responder<'r, 'static> for RequestError {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        let error_string = format!("{}", self);
        Response::build()
            .sized_body(error_string.len(), Cursor::new(error_string))
            .status(rocket::http::Status::BadRequest)
            .ok()
    }
}

async fn get_tts_url(text: &str) -> Result<String, RequestError> {
    let request_url = format!("http://tts.cyzon.us/tts?text={}", text);

    // Build a new request to grab what the tts url redirects to
    let timeout = Duration::new(5, 0);
    let client = ClientBuilder::new().timeout(timeout).build()?;
    let response = again::retry(|| client.head(&request_url).send()).await?;
    Ok(response.url().to_string())
}

#[get("/tts_lookup?<text>")]
async fn tts_lookup(text: &str ) -> Result<String, RequestError> {
    get_tts_url(text).await
}

#[get("/tts_redir?<text>")]
async fn tts_redir(text: &str ) -> Result<Redirect, RequestError> {
    let url = get_tts_url(text).await?;
    Ok(Redirect::to(url))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![tts_lookup, tts_redir])
}
