use cratesio;
use url::percent_encoding::{utf8_percent_encode, PATH_SEGMENT_ENCODE_SET};
use super::{Module, Flow, Context};

pub struct CrateInfo {
    prefix: String,
}

impl CrateInfo {
    pub fn new<S: Into<String>>(prefix: S) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }
}

impl Module for CrateInfo {
    fn run(&mut self, ctx: Context) -> Flow {
        let mut body = ctx.body();

        // Ensure the prefix exists
        if !body.starts_with(&self.prefix) {
            return Flow::Continue;
        }
        body = &body[self.prefix.len()..];

        // Ensure prefix is followed by at least one space
        if !body.starts_with(" ") {
            return Flow::Continue;
        }
        body = body.trim_left();

        let crate_name = match body.split_whitespace().next() {
            Some(crate_name) => crate_name,
            None => return Flow::Break,
        };

        let info = match cratesio::crate_info(crate_name) {
            Ok(info) => info,
            Err(_) => return Flow::Break,
        };

        let krate = info.krate();
        let output = format!(
            "{name} ({version}) - {description} -> https://crates.io/crates/{urlname} [https://docs.rs/crate/{urlname}]",
            name = krate.name(),
            version = krate.max_version(),
            description = krate.description(),
            urlname = utf8_percent_encode(&krate.name(), PATH_SEGMENT_ENCODE_SET).collect::<String>()
        );

        ctx.reply(output);

        Flow::Break
    }
}
