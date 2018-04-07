use playground::{self, ExecuteRequest, Channel, Mode};
use reqwest::Client;
use {Flow, Context};
use regex::Regex;

lazy_static! {
    static ref CRATE_ATTRS: Regex = Regex::new(r"^(\s*#!\[.*?\])*").unwrap();
}

pub fn handler(http: &Client) -> impl Fn(&Context) -> Flow {
    let http = http.clone();
    move |ctx| {
        if !ctx.is_directly_addressed() {
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
                "help" | "h" | "-h" | "--help" | "--h" => {
                    super::help::display_help(ctx);
                    return Flow::Break;
                }
                _ => break,
            }

            body = &body[flag.len()..];
        }

        if show_version {
            print_version(&http, channel, &ctx);
            return Flow::Break;
        }



        let code = if bare { body.to_string() } else {
            let crate_attrs = CRATE_ATTRS.find(body)
                .map(|attr| attr.as_str())
                .unwrap_or("");

            body = &body[crate_attrs.len()..];

            format!(include!("../../template.rs"),
                crate_attrs = crate_attrs,
                code = body,
            )
        };

        let mut request = ExecuteRequest::new(code.as_ref());
        request.set_channel(channel);
        request.set_mode(mode);

        execute(&ctx, &http, &request);

        Flow::Break
    }
}

fn print_version(http: &Client, channel: Channel, ctx: &Context) {
    let resp = match playground::version(http, channel) {
        Err(e) => return eprintln!("Failed to get version: {:?}", e),
        Ok(resp) => resp,
    };
    
    let version = format!("{version} ({hash:.9} {date})",
        version = resp.version,
        hash = resp.hash,
        date = resp.date,
    );

    ctx.reply(version);
}

pub fn execute(ctx: &Context, http: &Client, request: &ExecuteRequest) {
    let resp = match playground::execute(http, &request) {
        Ok(resp) => resp,
        Err(e) => return {
            eprintln!("Failed to execute code: {:?}", e);
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

        let url = match playground::paste(http, code, request.channel(), request.mode()) {
            Ok(url) => url,
            Err(e) => return {
                eprintln!("Failed to paste code: {:?}", e);
            },
        };

        ctx.reply(format!("~~~ Output truncated; full output at {}", url));
    }
}
