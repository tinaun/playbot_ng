use irc::client::prelude::{Config, IrcServer, Server, ServerExt, Command, ChannelExt};
use failure::{Error, SyncFailure};

pub fn run() -> Result<(), Error> {
    let config = Config {
        nickname: Some("eval".into()),
        nick_password: Some(option_env!("PASS").expect("PASS").into()),
        channels: Some(vec![
            "#rust".into(),
            "#rust-beginners".into(),
            "#rust-offtopic".into(),
            "#rust-de".into(),
            "#rust-fr".into(),
        ]),
        use_ssl: Some(true),
        server: Some("irc.mozilla.org".into()),
        port: Some(6697),
        encoding: Some("UTF-8".into()),
        should_ghost: Some(true),
        ghost_sequence: Some(vec!["RECOVER".into()]),
        burst_window_length: Some(3),
        max_messages_in_burst: Some(4),
        .. Config::default()
    };
    let server = IrcServer::from_config(config)
        .map_err(SyncFailure::new)?;
    let http = ::reqwest::Client::new();
    
    server.identify()
        .map_err(SyncFailure::new)?;

    server.for_each_incoming(|message| {
        let body = match message.command {
            Command::PRIVMSG(_, ref body) => body,
            _ => return,
        };

        let target = message.response_target().expect("response target");

        let current_nickname = server.current_nickname();

        let code = if !target.is_channel_name() {
            body.as_str()
        } else {
            if !body.starts_with(format!("{}:", current_nickname).as_str()) {
                return;
            }

            body[current_nickname.len()+1..].trim()
        };

        let resp = match ::execute(&http, code) {
            Err(e) => return eprintln!("Failed to execute code: {:?}", e),
            Ok(resp) => resp,
        };

        let output = if resp.success { resp.stdout } else { resp.stderr };

        let send = |msg| if target.is_channel_name() {
            server.send_notice(&target, msg)
        } else {
            server.send_privmsg(&target, msg)
        };

        for line in output.lines().take(2) {
            if let Err(e) = send(line) {
                eprintln!("Failed to send message: {:?}", e);
            }
        }

        if output.lines().count() > 2 {
            if let Err(e) = send("~~~ output truncated ~~~") {
                eprintln!("Failed to send message: {:?}", e);
            }
        }
    }).map_err(SyncFailure::new)?;

    Ok(())
}