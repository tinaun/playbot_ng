#![feature(generators)]
#![feature(pin)]
#![feature(proc_macro)]

extern crate failure;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate apply;

extern crate futures_await as futures;
extern crate futures_adapter;

use std::str;

pub mod execute;
pub use execute::{
    execute,
    Request as ExecuteRequest,
    Response as ExecuteResponse,
};

mod version;
pub use version::{version, Version};

pub mod paste;
pub use paste::paste;

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

#[derive(Serialize,Debug,Clone)]
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

#[derive(Deserialize,Clone)]
pub struct Crates {
    crates: Vec<Crate>,
}

#[derive(Deserialize,Clone)]
pub struct Crate {
    name: String,
    version: String,
    id: String,
}
