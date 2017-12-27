extern crate serde;
extern crate failure;
extern crate irc;
extern crate reqwest;
#[macro_use] extern crate serde_derive;

mod playground;
mod bot;

use playground::{ExecuteRequest, ExecuteResponse};
use failure::Error;
use reqwest::Client;

fn main() {
    loop {   
        if let Ok(e) = bot::run() {
            eprintln!("Disconnected because: {:?}", e);
        } else {
            eprintln!("Disconnected for an unknown reason");
        }
    }
}

fn execute(client: &Client, code: &str) -> Result<ExecuteResponse, Error> {
    let code = format!(include!("../template.rs"), code = code);
    let req = ExecuteRequest::new(code);

    playground::execute(client, &req)
}
