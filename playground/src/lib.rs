extern crate failure;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;

use std::str;

pub mod execute;
pub use execute::execute;

pub mod version;
pub use version::{version, Version};

#[derive(Serialize,Debug,Copy,Clone)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Debug,
    Release,
}

impl Mode {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Mode::Debug => "debug",
            Mode::Release => "release",
        }
    }
}

#[derive(Serialize,Debug)]
#[serde(rename_all = "lowercase")]
pub enum CrateType {
    Bin,
}

#[derive(Serialize,Debug,Copy,Clone)]
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
pub struct Crates {
    crates: Vec<Crate>,
}

#[derive(Deserialize)]
pub struct Crate {
    name: String,
    version: String,
    id: String,
}
