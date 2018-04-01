use {Channel, Mode};
use reqwest::unstable::async::Client;
use reqwest::Error;
use futures::prelude::*;
use futures_adapter::OldFuture;
use apply::Apply;

#[async]
pub fn paste(
    client: Client,
    text: String,
    channel: Channel,
    mode: Mode,
) -> Result<String, Error> {
    let resp = client
        .post("https://play.rust-lang.org/meta/gist/")
        .json(&Request::new(text))
        .send()
        .apply(OldFuture);
    let mut resp = await!(resp)?.error_for_status()?;
    let resp = resp.json::<Response>().apply(OldFuture);
    let gist_id = await!(resp)?.id;

    let url = format!(
        "https://play.rust-lang.org/?gist={gist}&version={channel}&mode={mode}",
        gist = gist_id,
        channel = channel.as_str(),
        mode = mode.as_str()
    );

    Ok(url)
}

#[derive(Serialize, Clone)]
struct Request {
    code: String,
}

impl Request {
    fn new(code: impl Into<String>) -> Self {
        Request {
            code: code.into()
        }
    }
}

#[derive(Deserialize)]
struct Response {
    id: String,
}
