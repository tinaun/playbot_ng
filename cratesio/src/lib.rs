#![feature(generators)]
#![feature(pin)]
#![feature(proc_macro)]

extern crate reqwest;
extern crate url;
#[macro_use]
extern crate serde_derive;
extern crate apply;
// extern crate futures_await as futures;

extern crate futures_await as futures;
extern crate futures_adapter;

use futures::prelude::*;
use url::percent_encoding::{utf8_percent_encode, PATH_SEGMENT_ENCODE_SET};
use reqwest::unstable::async::Client;
use apply::Apply;
use futures_adapter::OldFuture;

pub fn crate_info(
    client: &Client,
    name: &str,
) -> impl Future<Item = Info, Error = reqwest::Error> {
    let client = client.clone();
    let url = format!(
        "https://crates.io/api/v1/crates/{}",
        utf8_percent_encode(name, PATH_SEGMENT_ENCODE_SET).collect::<String>()
    );

    async_block! {

        let resp = client.get(&url).send().apply(OldFuture);
        let json = await!(resp)?.json().apply(OldFuture);

        await!(json)
    }
}


#[derive(Deserialize,Debug,Clone,PartialEq,Eq)]
pub struct Info {
    #[serde(rename = "crate")]
    krate: Crate,
}

#[derive(Deserialize,Debug,Clone,PartialEq,Eq)]
pub struct Crate {
    id: String,
    name: String,
    description: String,
    max_version: String,
}

impl Info {
    pub fn krate(&self) -> &Crate {
        &self.krate
    }
}

impl Crate {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn max_version(&self) -> &str {
        &self.max_version
    }
}
