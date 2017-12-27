use failure::Error;
use reqwest::Client;
use std::str;

pub fn execute(client: &Client, req: &ExecuteRequest) -> Result<ExecuteResponse, Error> {
    let resp = client
        .post("https://play.rust-lang.org/execute")
        .json(req)
        .send()?
        .json()?;
    
    Ok(resp)
}

#[derive(Serialize,Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteRequest {
    code: String,
    channel: Channel,
    crate_type: CrateType,
    mode: Mode,
    tests: bool,
}

impl ExecuteRequest {
    pub fn new<S: Into<String>>(code: S) -> Self {
        Self {
            code: code.into(),
            channel: Channel::Stable,
            crate_type: CrateType::Bin,
            mode: Mode::Debug,
            tests: false,
        }
    }
}

#[derive(Deserialize,Debug)]
pub struct ExecuteResponse {
    pub stderr: String,
    pub stdout: String,
    pub success: bool,
}

#[derive(Serialize,Debug)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Debug,
    Release,
}

#[derive(Serialize,Debug)]
#[serde(rename_all = "lowercase")]
pub enum CrateType {
    Bin,
}

#[derive(Serialize,Debug)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Stable,
    Beta,
    Nightly,
}

impl Channel {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Channel::Stable => "stable",
            Channel::Beta => "beta",
            Channel::Nightly => "nightly",
        }
    }
}

#[derive(Deserialize)]
pub struct Version {
    date: String,
    hash: String,
    version: String,
}

#[derive(Deserialize)]
pub struct Crates {
    crates: Vec<Crate>,
}

#[derive(Deserialize)]
pub struct Crate {
    name: String,
    version: String,
    id: String,
}
