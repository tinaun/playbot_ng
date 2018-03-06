use irc::client::prelude::{ChannelExt, Command, IrcServer, Message, Server, ServerExt};
use failure::{Error, SyncFailure};
use reqwest;
use irc;

mod crate_info;
mod playground;
mod codedb;

use self::playground::Playground;
use self::crate_info::CrateInfo;
use self::codedb::CodeDB;

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

    let server = IrcServer::new("config.toml").map_err(SyncFailure::new)?;
    let http = reqwest::Client::new();

    server.identify().map_err(SyncFailure::new)?;

    let mut modules = vec![
        CrateInfo::new("?crate").boxed(),
        //        CodeDB::new(&mut codedb, &http).boxed(),
        Playground::new(&http).boxed(),
    ];

    server
        .for_each_incoming(|message| {
            let context = match Context::new(&server, &message) {
                Some(context) => context,
                None => return,
            };

            if context.is_ctcp() {
                return;
            }

            if modules
                .iter_mut()
                .any(|module| module.run(context.clone()) == Flow::Break)
            {
                return;
            }
        })
        .map_err(SyncFailure::new)?;

    Ok(())
}

#[derive(Clone)]
pub struct Context<'a> {
    body: &'a str,
    is_directly_addressed: bool,
    is_ctcp: bool,
    send_fn: fn(&IrcServer, &str, &str) -> irc::error::Result<()>,
    source: &'a str,
    target: &'a str,
    server: &'a IrcServer,
}

impl<'a> Context<'a> {
    pub fn new(server: &'a IrcServer, message: &'a Message) -> Option<Self> {
        let mut body = match message.command {
            Command::PRIVMSG(_, ref body) => body.trim(),
            _ => return None,
        };

        let is_ctcp = body.len() >= 2 && &body[..1] == "\x01" && &body[body.len() - 1..] == "\x01";

        if is_ctcp {
            body = &body[1..body.len() - 1];
        }

        let source = message
            .prefix
            .as_ref()
            .map(<_>::as_ref)
            .unwrap_or("<unknown>");

        let target = match message.response_target() {
            Some(target) => target,
            None => {
                eprintln!("Unknown response target");
                return None;
            }
        };

        let is_directly_addressed = {
            let current_nickname = server.current_nickname();

            if body.starts_with(current_nickname) {
                let new_body = body[current_nickname.len()..].trim_left();

                if new_body.starts_with(":") || new_body.starts_with(",") {
                    body = new_body[1..].trim_left();
                    true
                } else {
                    false
                }
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
            source,
            target,
            is_directly_addressed,
            is_ctcp,
        })
    }

    pub fn body(&self) -> &str {
        self.body
    }

    /// Wether the message was aimed directetly at the bot,
    /// either via private message or by prefixing a channel message with
    /// the bot's name, followed by ',' or ':'.
    pub fn is_directly_addressed(&self) -> bool {
        self.is_directly_addressed
    }

    pub fn is_ctcp(&self) -> bool {
        self.is_ctcp
    }

    pub fn reply<S: AsRef<str>>(&self, message: S) {
        (self.send_fn)(self.server, self.target, message.as_ref());
    }

    pub fn source(&self) -> &str {
        self.source
    }
}
