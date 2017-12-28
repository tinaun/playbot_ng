use irc::client::prelude::{Config, IrcServer, Server, ServerExt, Command, ChannelExt};
use failure::{Error, SyncFailure};
use playground::{self, ExecuteRequest, Channel};
use paste::paste;

pub fn run() -> Result<(), Error> {
    let server = IrcServer::new("config.toml")
        .map_err(SyncFailure::new)?;
    let http = ::reqwest::Client::new();

    server.identify()
        .map_err(SyncFailure::new)?;

    server.for_each_incoming(|message| {
        let body = match message.command {
            Command::PRIVMSG(_, ref body) => body.trim(),
            _ => return,
        };

        let target = message.response_target().expect("response target");

        let current_nickname = server.current_nickname();

        let code = if !target.is_channel_name() {
            body
        } else {
            if !body.starts_with(&format!("{}:", current_nickname)) {
                return;
            }

            body[current_nickname.len()+1..].trim_left()
        };

        let (channel, code) = if code.starts_with("--nightly ") {
            (Channel::Nightly, code[10..].trim_left())
        } else if code.starts_with("--beta ") {
            (Channel::Beta, code[7..].trim_left())
        } else {
            (Channel::Stable, code)
        };

        let code = format!(include!("../template.rs"), code = code);

        let mut request = ExecuteRequest::new(code.as_ref());
        request.set_channel(channel);

        let resp = match playground::execute(&http, &request) {
            Err(e) => return eprintln!("Failed to execute code: {:?}", e),
            Ok(resp) => resp,
        };

        let output = if resp.success { &resp.stdout } else { &resp.stderr };

        let send = |msg: &str| if target.is_channel_name() {
            server.send_notice(&target, msg)
        } else {
            server.send_privmsg(&target, msg)
        };

        let skip_count = if resp.success { 0 } else { 1 };

        for line in output.lines().skip(skip_count).take(2) {
            if let Err(e) = send(line) {
                eprintln!("Failed to send message: {:?}", e);
            }
        }

        if output.lines().count() > 2 {
            let code = format!(include!("../paste_template.rs"),
                code = code,
                stdout = resp.stdout,
                stderr = resp.stderr,
            );

            let url = match paste(&http, code) {
                Ok(url) => url,
                Err(e) => return eprintln!("Failed to paste code: {:?}", e),
            };

            if let Err(e) = send(&format!("~~~ Output truncated; full output at {}.rs", url)) {
                eprintln!("Failed to send message: {:?}", e);
            }
        }
    }).map_err(SyncFailure::new)?;

    Ok(())
}