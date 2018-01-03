#![feature(box_patterns)]
#![feature(nll)]
extern crate serde;
#[macro_use] extern crate failure;
extern crate irc;
extern crate reqwest;
extern crate syn;
extern crate url;
#[macro_use] extern crate serde_derive;
extern crate serde_json as json;
extern crate chrono;

mod playground;
mod paste;
mod cratesio;
mod codedb;
mod bot;

fn main() {
    loop {   
        if let Ok(e) = bot::run() {
            eprintln!("Disconnected because: {:?}", e);
        } else {
            eprintln!("Disconnected for an unknown reason");
        }
    }
}
