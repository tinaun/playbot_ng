use irc::client::prelude::{IrcServer, Server, ServerExt, Command, ChannelExt, Message};
use failure::{Error, SyncFailure};
use reqwest;
use irc;

mod crate_info;
mod playground;

use self::playground::Playground;
use self::crate_info::CrateInfo;

pub trait Module {
    fn run(&mut self, ctx: Context) -> Flow;
    fn boxed<'a>(self) -> Box<Module + 'a> where Self: Sized + 'a {
        Box::new(self)
    }
}

#[derive(PartialEq, Eq)]
pub enum Flow {
    Break,
    Continue,
}

pub fn run() -> Result<(), Error> {
    let server = IrcServer::new("config.toml")
        .map_err(SyncFailure::new)?;
    let http = reqwest::Client::new();

    server.identify()
        .map_err(SyncFailure::new)?;

    let mut modules = vec![
        CrateInfo::new("?crate").boxed(),
        Playground::new(&http).boxed(),
    ];

    server.for_each_incoming(|message| {
        let context = match Context::new(&server, &message) {
            Some(context) => context,
            None => return,
        };

        if modules.iter_mut().any(|module| module.run(context.clone()) == Flow::Break) {
            return;
        }
    }).map_err(SyncFailure::new)?;

    Ok(())
}

#[derive(Clone)]
pub struct Context<'a> {
    body: &'a str,
    directly_addressed: bool,
    send_fn: fn(&IrcServer, &str, &str) -> irc::error::Result<()>,
    target: &'a str,
    server: &'a IrcServer,
}

impl<'a> Context<'a> {
    pub fn new(server: &'a IrcServer, message: &'a Message) -> Option<Self> {
        let mut body = match message.command {
            Command::PRIVMSG(_, ref body) => body.trim(),
            _ => return None,
        };

        let target = match message.response_target() {
            Some(target) => target,
            None => {
                eprintln!("Unknown response target");
                return None;
            },
        };

        let directly_addressed = {
            let current_nickname = server.current_nickname();
            if body.starts_with(&format!("{}:", current_nickname)) {
                body = body[current_nickname.len()+1..].trim_left();
                true
            } else {
                !target.is_channel_name()
            }
        };

        let send_fn = match target.is_channel_name() {
            true => IrcServer::send_notice,
            false => IrcServer::send_privmsg,
        };

        Some(Self {
            server,
            body,
            send_fn,
            target,
            directly_addressed,
        })
    }

    pub fn body(&self) -> &str {
        self.body
    }

    pub fn directly_addressed(&self) -> bool {
        self.directly_addressed
    }

    pub fn reply<S: AsRef<str>>(&self, message: S) {
        (self.send_fn)(self.server, self.target, message.as_ref());
    }
}
