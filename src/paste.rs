use reqwest::Client;
use failure::Error;
use playground::{Channel, Mode};

use std::collections::HashMap;

/*
pub fn paste<S: Into<String>>(client: &Client, text: S) -> Result<String, Error> {
    let url = client
        .post("https://paste.rs")
        .body(text.into())
        .send()?
        .error_for_status()?
        .text()?
        .trim()
        .to_owned();
    
    Ok(url)
}
*/

pub fn paste<S: AsRef<str>>(client: &Client, text: S, channel: Channel, mode: Mode) -> Result<String, Error> {
    let gist_id = client
        .post("https://api.github.com/gists")
        .json(&Request::new(text.as_ref()))
        .send()?
        .error_for_status()?
        .json::<Response>()?
        .id;

    let url = format!("https://play.rust-lang.org/?gist={gist}&version={channel}&mode={mode}",
        gist = gist_id,
        channel = channel.as_str(),
        mode = mode.as_str()
    );

    Ok(url)
}

#[derive(Serialize)]
struct Request<'a> {
    files: HashMap<&'static str, File<'a>>,
}

impl<'a> Request<'a> {
    fn new(text: &'a str) -> Self {
        let mut files = HashMap::new();
        files.insert("main.rs", File { content: text.as_ref() });
        Request { files }
    }
}

#[derive(Serialize)]
struct File<'a> {
    content: &'a str,
}

#[derive(Deserialize)]
struct Response {
    id: String,
}
