#![feature(box_patterns)]
#![feature(option_filter)]
extern crate failure;
extern crate irc;
extern crate reqwest;
extern crate url;
extern crate chrono;
extern crate itertools;
extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate playground;
extern crate cratesio;

use chrono::prelude::*;

// mod codedb;
mod bot;

fn main() {
    loop {   
        println!("{} Starting up", Utc::now());
        if let Ok(e) = bot::run() {
            eprintln!("Disconnected because: {:?}", e);
        } else {
            eprintln!("Disconnected for an unknown reason");
        }
        println!("{} Terminated", Utc::now());
    }
}
