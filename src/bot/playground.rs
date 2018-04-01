use playground::{self, ExecuteRequest, Channel, Mode};
use reqwest::unstable::async::Client;
use super::{Flow, Context, FlowFuture};
use std::sync::Arc;

use futures::prelude::*;

pub fn handler(http: Client) -> Arc<Fn(Context) -> FlowFuture> {
    Arc::new(move |ctx| {
        let http = http.clone();
// 
        Box::new(async_block! {

            if !ctx.is_directly_addressed() {
                return Ok(Flow::Continue);
            }

            let mut body = ctx.body().clone();
            let mut channel = Channel::Stable;
            let mut show_version = false;
            let mut bare = false;
            let mut mode = Mode::Debug;

            // Parse flags
            loop {
                body = body.trim_left();
                let flag = body.split_whitespace().next().unwrap_or(body.from_str(""));

                match flag.as_str() {
                    "--stable" => channel = Channel::Stable,
                    "--beta" => channel = Channel::Beta,
                    "--nightly" => channel = Channel::Nightly,
                    "--version" | "VERSION" => show_version = true,
                    "--bare" | "--mini" => bare = true,
                    "--debug" => mode = Mode::Debug,
                    "--release" => mode = Mode::Release,
                    _ => break,
                }

                body = body.slice(flag.len()..);
            }

            if show_version {
                await!(print_version(http.clone(), channel, ctx.clone()));
                return Ok(Flow::Break);
            }

            let code = if bare { body.to_string() } else {
                format!(include!("../../template.rs"), code = body)
            };

            let mut request = ExecuteRequest::new(code);
            request.set_channel(channel);
            request.set_mode(mode);

            await!(execute(ctx, http, request));

            Ok(Flow::Break)
        })
    })
}

#[async] 
fn print_version(http: Client, channel: Channel, ctx: Context) -> Result<(), ()> {
    let resp = match await!(playground::version(http, channel)) {
        Err(e) => {
            eprintln!("Failed to get version: {:?}", e);
            return Ok(());
        }
        Ok(resp) => resp,
    };
    
    let version = format!("{version} ({hash:.9} {date})",
        version = resp.version,
        hash = resp.hash,
        date = resp.date,
    );

    ctx.reply(&version);

    Ok(())
}

#[async]
pub fn execute(ctx: Context, http: Client, request: ExecuteRequest) -> Result<(), ()> {
    let resp = match await!(playground::execute(http.clone(), request.clone())) {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Failed to execute code: {:?}", e);
            return Ok(())
        },
    };

    let output = if resp.success { &resp.stdout } else { &resp.stderr };

    let skip_count = if resp.success { 0 } else { 1 };

    for line in output.lines().skip(skip_count).take(2) {
        ctx.reply(line);
    }

    if output.lines().count() > 2 {
        let code = format!(include!("../../paste_template.rs"),
            code = request.code(),
            stdout = resp.stdout,
            stderr = resp.stderr,
        );

        let url = match await!(playground::paste(http, code, request.channel(), request.mode())) {
            Ok(url) => url,
            Err(e) => {
                eprintln!("Failed to paste code: {:?}", e);
                return Ok(());
            },
        };

        ctx.reply(&format!("~~~ Output truncated; full output at {}", url));
    }

    Ok(())
}
