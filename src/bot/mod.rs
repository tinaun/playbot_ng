use irc::client::prelude::{Config, IrcReactor, ClientExt};
use failure::Error;
use reqwest;

mod context;
mod command;
mod command_registry;
mod crate_info;
mod playground;
// mod codedb;
mod egg;

use self::context::Context;
use self::command::Command;
use self::command_registry::CommandRegistry;
// use self::codedb::CodeDB;
use futures::prelude::*;
use futures_adapter::NewFuture;
use apply::Apply;

pub type FlowFuture = Box<Future<Item = Flow, Error = ()>>;

pub fn run() -> Result<(), Error> {
    //    let mut codedb = ::codedb::CodeDB::open_or_create("code_db.json")?;
    let mut reactor = IrcReactor::new()?;
    let config = Config::load("config.toml")?;
    let client = reactor.prepare_client_and_connect(&config)?;
    let http = reqwest::unstable::async::Client::new(&reactor.inner_handle());
    let mut commands = CommandRegistry::new("?");

    commands.set_named_handler("crate", crate_info::handler(http.clone()));
    commands.add_fallback_handler(egg::handler());
    commands.add_fallback_handler(playground::handler(http.clone()));

    client.identify()?;

    reactor
        .register_client_with_handler(client, move |client, message| {
            commands
                .handle_message(client, message)
                .apply(NewFuture)
        });

    // reactor blocks until a disconnection or other in `irc` error
    reactor.run()?;

    Ok(())
}

#[derive(PartialEq, Eq)]
pub enum Flow {
    Break,
    Continue,
}
