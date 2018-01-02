use playground::{self, ExecuteRequest, Channel, Mode};
use paste::paste;
use reqwest::Client;
use super::{Module, Flow, Context};

pub struct Playground<'a> {
    http: &'a Client,
}

impl<'a> Playground<'a> {
    pub fn new(http: &'a Client) -> Self {
        Self { http }
    }
}

impl<'a> Module for Playground<'a> {
    fn run(&mut self, ctx: Context) -> Flow {
        if !ctx.directly_addressed() {
            return Flow::Continue;
        }

        let mut body = ctx.body();
        let mut channel = Channel::Stable;
        let mut show_version = false;
        let mut bare = false;
        let mut mode = Mode::Debug;

        // Parse flags
        loop {
            body = body.trim_left();
            let flag = body.split_whitespace().next().unwrap_or("");

            match flag {
                "--stable" => channel = Channel::Stable,
                "--beta" => channel = Channel::Beta,
                "--nightly" => channel = Channel::Nightly,
                "--version" | "VERSION" => show_version = true,
                "--bare" | "--mini" => bare = true,
                "--debug" => mode = Mode::Debug,
                "--release" => mode = Mode::Release,
                _ => break,
            }

            body = &body[flag.len()..];
        }

        if show_version {
            print_version(self.http, channel, &ctx);
            return Flow::Break;
        }

        let code = if bare { body.to_string() } else {
            format!(include!("../../template.rs"), code = body)
        };

        let mut request = ExecuteRequest::new(code.as_ref());
        request.set_channel(channel);
        request.set_mode(mode);

        let resp = match playground::execute(&self.http, &request) {
            Ok(resp) => resp,
            Err(e) => return {
                eprintln!("Failed to execute code: {:?}", e);
                Flow::Break
            },
        };

        let output = if resp.success { &resp.stdout } else { &resp.stderr };

        let skip_count = if resp.success { 0 } else { 1 };

        for line in output.lines().skip(skip_count).take(2) {
            ctx.reply(line);
        }

        if output.lines().count() > 2 {
            let code = format!(include!("../../paste_template.rs"),
                code = code,
                stdout = resp.stdout,
                stderr = resp.stderr,
            );

            let url = match paste(self.http, code) {
                Ok(url) => url,
                Err(e) => return {
                    eprintln!("Failed to paste code: {:?}", e);
                    Flow::Break
                },
            };

            ctx.reply(&format!("~~~ Output truncated; full output at {}.rs", url));
        }

        Flow::Break
    }
}

fn print_version(http: &Client, channel: Channel, ctx: &Context) {
    let resp = match ::playground::version(http, channel) {
        Err(e) => return eprintln!("Failed to get version: {:?}", e),
        Ok(resp) => resp,
    };
    
    let version = format!("{version} ({hash:.9} {date})",
        version = resp.version,
        hash = resp.hash,
        date = resp.date,
    );

    ctx.reply(&version);
}
