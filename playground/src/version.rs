use Channel;
use reqwest::unstable::async::Client;
use reqwest::Error;
use futures::prelude::*;
use futures_adapter::OldFuture;
use apply::Apply;

#[async]
pub fn version(
    client: Client,
    channel: Channel,
) -> Result<Version, Error> {
    let resp = client
        .get(&format!("https://play.rust-lang.org/meta/version/{}", channel.as_str()))
        .send()
        .apply(OldFuture);
    let mut resp = await!(resp)?.error_for_status()?;
    let json = resp.json().apply(OldFuture);

    await!(json)
}


#[derive(Deserialize)]
pub struct Version {
    pub date: String,
    pub hash: String,
    pub version: String,
}
