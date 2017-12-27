use reqwest::Client;
use failure::Error;

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
