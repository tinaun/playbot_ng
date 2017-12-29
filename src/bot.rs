use irc::client::prelude::{Config, IrcServer, Server, ServerExt, Command, ChannelExt, Message};
use failure::{Error, SyncFailure};
use playground::{self, ExecuteRequest, Channel};
use paste::paste;
use reqwest;
use irc;

pub fn run() -> Result<(), Error> {
    let server = IrcServer::new("config.toml")
        .map_err(SyncFailure::new)?;
    let http = reqwest::Client::new();

    server.identify()
        .map_err(SyncFailure::new)?;

    server.for_each_incoming(|message| {
        let context = match Context::new(&http, &server, &message) {
            Some(context) => context,
            None => return,
        };

        if context.show_version {
            show_version(&context);
        } else {
            execute_code(&context);
        }
    }).map_err(SyncFailure::new)?;

    Ok(())
}

struct Context<'a> {
    server: &'a IrcServer,
    http: &'a reqwest::Client,
    send_fn: fn(&IrcServer, &str, &str) -> irc::error::Result<()>,
    target: &'a str,
    body: &'a str,
    channel: Channel,
    show_version: bool,
}

impl<'a> Context<'a> {
    fn new(http: &'a reqwest::Client, server: &'a IrcServer, message: &'a Message) -> Option<Self> {
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
        let send_fn = match target.is_channel_name() {
            true => IrcServer::send_notice,
            false => IrcServer::send_privmsg,
        };

        // Strip bot address
        {
            let current_nickname = server.current_nickname();
            if target.is_channel_name() && body.starts_with(&format!("{}:", current_nickname)) {
                body = body[current_nickname.len()+1..].trim_left();
            }
        }

        let mut channel = Channel::Stable;
        let mut show_version = false;

        // Parse flags
        loop {
            body = body.trim_left();
            let flag = body.split_whitespace().next().unwrap_or("");

            match flag {
                "--stable" => channel = Channel::Stable,
                "--beta" => channel = Channel::Beta,
                "--nightly" => channel = Channel::Nightly,
                "--version" => show_version = true,
                _ => break,
            }

            body = &body[flag.len()..];
        }

        Some(Context {
            http,
            server,
            show_version,
            target,
            send_fn,
            channel,
            body,
        })
    }

    fn reply(&self, message: &str) -> Result<(), Error> {
        (self.send_fn)(self.server, self.target, message)
            .map_err(SyncFailure::new)
            .map_err(Error::from)
    }
}

fn show_version(context: &Context) {
    let resp = match playground::version(&context.http, context.channel) {
        Err(e) => return eprintln!("Failed to get version: {:?}", e),
        Ok(resp) => resp,
    };
    
    let version = format!("{version} ({hash:.9} {date})",
        version = resp.version,
        hash = resp.hash,
        date = resp.date,
    );

    if let Err(e) = context.reply(&version) {
        eprintln!("Failed to send message: {:?}", e);
    }
}

fn execute_code(context: &Context) {
    let code = format!(include!("../template.rs"), code = context.body);

    let mut request = ExecuteRequest::new(code.as_ref());
    request.set_channel(context.channel);

    let resp = match playground::execute(&context.http, &request) {
        Err(e) => return eprintln!("Failed to execute code: {:?}", e),
        Ok(resp) => resp,
    };

    let output = if resp.success { &resp.stdout } else { &resp.stderr };

    let skip_count = if resp.success { 0 } else { 1 };

    for line in output.lines().skip(skip_count).take(2) {
        if let Err(e) = context.reply(line) {
            eprintln!("Failed to send message: {:?}", e);
        }
    }

    if output.lines().count() > 2 {
        let code = format!(include!("../paste_template.rs"),
            code = code,
            stdout = resp.stdout,
            stderr = resp.stderr,
        );

        let url = match paste(&context.http, code) {
            Ok(url) => url,
            Err(e) => return eprintln!("Failed to paste code: {:?}", e),
        };

        if let Err(e) = context.reply(&format!("~~~ Output truncated; full output at {}.rs", url)) {
            eprintln!("Failed to send message: {:?}", e);
        }
    }
}
