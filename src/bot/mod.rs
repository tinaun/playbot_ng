use irc::client::prelude::{Config, ChannelExt, Command, IrcReactor, IrcClient, Message, ClientExt};
use failure::Error;
use reqwest;
use irc;

mod context;
mod crate_info;
mod playground;
mod codedb;
mod egg;

use self::context::Context;
use self::playground::Playground;
use self::crate_info::CrateInfo;
use self::codedb::CodeDB;
use self::egg::Egg;

pub trait Module {
    fn run(&mut self, ctx: Context) -> Flow;
    fn boxed<'a>(self) -> Box<Module + 'a>
    where
        Self: Sized + 'a,
    {
        Box::new(self)
    }
}

#[derive(PartialEq, Eq)]
pub enum Flow {
    Break,
    Continue,
}

pub fn run() -> Result<(), Error> {
    //    let mut codedb = ::codedb::CodeDB::open_or_create("code_db.json")?;

    let mut reactor = IrcReactor::new()?;
    let config = Config::load("config.toml")?;
    let client = reactor.prepare_client_and_connect(&config)?;
    let http = reqwest::Client::new();

    client.identify()?;

    let mut modules = vec![
        CrateInfo::new("?crate").boxed(),
        //        CodeDB::new(&mut codedb, &http).boxed(),
        Egg::new().boxed(),
        Playground::new(http).boxed(),
    ];

    reactor
        .register_client_with_handler(client, move |client, message| {
            let context = match Context::new(&client, &message) {
                Some(context) => context,
                None => return Ok(()),
            };

            if context.is_ctcp() {
                return Ok(());
            }

            if modules
                .iter_mut()
                .any(|module| module.run(context.clone()) == Flow::Break)
            {
                return Ok(());
            }
            
            Ok(())
        });

    // reactor blocks until a disconnection or other in `irc` error
    reactor.run()?;

    Ok(())
}
