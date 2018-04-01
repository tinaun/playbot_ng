#![feature(box_patterns)]
#![feature(option_filter)]
#![feature(generators)]
#![feature(pin)]
#![feature(proc_macro)]
// #![feature(nll)]
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
extern crate apply;
extern crate shared_str;

extern crate futures_await as futures;
extern crate futures_adapter;

use chrono::prelude::*;
use chrono::Duration;
use std::thread;

// mod codedb;
mod bot;

fn main() {
    let sleep_dur = Duration::seconds(5).to_std().unwrap();

    loop {   
        println!("{} Starting up", Utc::now());

        match bot::run() {
            Ok(()) => eprintln!("[OK] Disconnected for an unknown reason"),
            Err(e) => {
                eprintln!("[ERR] Disconnected");

                for cause in e.causes() {
                    eprintln!("[ERR] Caused by: {}", cause);
                }
            }
        }

        eprintln!("Reconnecting in 5 seconds");

        thread::sleep(sleep_dur);

        println!("{} Terminated", Utc::now());
    }
}
