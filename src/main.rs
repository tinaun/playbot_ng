#![feature(box_patterns)]
#![feature(option_filter)]
extern crate failure;
extern crate serenity;
extern crate toml;
extern crate threadpool;
extern crate reqwest;
extern crate url;
extern crate chrono;
extern crate itertools;
extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate playground;
extern crate cratesio;

use std::thread;
use chrono::{
    prelude::*,
    Duration,
};
use serenity::prelude::{Client, EventHandler};
use failure::{Error, SyncFailure};
use self::{
    context::Context,
    command::Command,
    command_registry::CommandRegistry,
};
use module::Module;
use config::Config;

mod context;
mod command;
mod command_registry;
mod module;
mod config;
// mod codedb;

struct Handler;

impl EventHandler for Handler {}

fn main() {
    let sleep_dur = Duration::seconds(5).to_std().unwrap();

    loop {   
        println!("{} Starting up", Utc::now());

        match run() {
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

pub fn run() -> Result<(), Error> {
    //    let mut codedb = ::codedb::CodeDB::open_or_create("code_db.json")?;

    let config = Config::load("config.toml")?;
    let mut client = Client::new(config.token(), Handler).map_err(|e| SyncFailure::new(e))?;
    
    let mut commands = CommandRegistry::new("?");

    module::CrateInfo::init(&mut commands);
    module::Help::init(&mut commands);
    module::Egg::init(&mut commands);
    module::Playground::init(&mut commands);

    client.with_framework(commands);

    // reactor blocks until a disconnection or other in `irc` error
    client.start().map_err(|e| SyncFailure::new(e))?;

    Ok(())
}

#[derive(PartialEq, Eq)]
pub enum Flow {
    Break,
    Continue,
}
