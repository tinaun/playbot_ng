use {Channel, CrateType, Mode};
use reqwest::unstable::async::Client;
use reqwest::Error;
use futures::prelude::*;
use futures_adapter::OldFuture;
use apply::Apply;

#[async]
pub fn execute(
    client: Client,
    req: Request,
) -> Result<Response, Error> {
    let resp = client
        .post("https://play.rust-lang.org/execute")
        .json(&req)
        .send()
        .apply(OldFuture);
    let mut resp = await!(resp)?.error_for_status()?;
    let resp = resp.json().apply(OldFuture);

    await!(resp)
}


#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    code: String,
    channel: Channel,
    crate_type: CrateType,
    mode: Mode,
    tests: bool,
}

impl Request {
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            channel: Channel::Stable,
            crate_type: CrateType::Bin,
            mode: Mode::Debug,
            tests: false,
        }
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn channel(&self) -> Channel {
        self.channel
    }

    pub fn set_channel(&mut self, channel: Channel) {
        self.channel = channel;
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
}

#[derive(Deserialize,Debug)]
pub struct Response {
    pub stderr: String,
    pub stdout: String,
    pub success: bool,
}
