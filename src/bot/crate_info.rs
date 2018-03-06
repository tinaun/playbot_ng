use cratesio;
use url::percent_encoding::{utf8_percent_encode, PATH_SEGMENT_ENCODE_SET};
use super::{Module, Flow, Context};
use itertools::Itertools;
use reqwest;
use reqwest::StatusCode::NotFound;

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
            // TODO: Use proper error types
            Err(err) => match err.downcast::<reqwest::Error>() {
                Ok(ref err) if err.status() == Some(NotFound) => {
                    ctx.reply(format!("Crate '{}' does not exist.", crate_name));
                    return Flow::Break
                },
                Ok(err) => {
                    eprintln!("Error getting crate info for '{}': {:?}", crate_name, err);
                    ctx.reply(format!("Failed to get crate info for {}", crate_name));
                    return Flow::Break
                },
                Err(err) => {
                    eprintln!("Error getting crate info for '{}': {:?}", crate_name, err);
                    ctx.reply(format!("Failed to get crate info for {}", crate_name));
                    println!("?crate {}: other error: {:?}", crate_name, err);
                    return Flow::Break
                }
            },
        };

        let krate = info.krate();
        let output = format!(
            "{name} ({version}) - {description} -> https://crates.io/crates/{urlname} [https://docs.rs/crate/{urlname}]",
            name = krate.name(),
            version = krate.max_version(),
            description = krate.description().split_whitespace().join(" "),
            urlname = utf8_percent_encode(&krate.name(), PATH_SEGMENT_ENCODE_SET).collect::<String>()
        );

        ctx.reply(output);

        Flow::Break
    }
}
