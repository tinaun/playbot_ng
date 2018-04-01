use cratesio;
use url::percent_encoding::{utf8_percent_encode, PATH_SEGMENT_ENCODE_SET};
use super::{Flow, Context, FlowFuture};
use itertools::Itertools;
use reqwest::StatusCode::NotFound;
use futures::prelude::*;
use reqwest::unstable::async::Client;
use shared_str::SharedStr;
use std::sync::Arc;

pub fn handler(client: Client) -> Arc<Fn(Context, &[SharedStr]) -> FlowFuture> {
    Arc::new(move |ctx, args| {
        let client = client.clone();
        let args = args.to_vec();
    
        Box::new(async_block! {
            let crate_name: SharedStr = match args.get(0).cloned() {
                Some(name) => name,
                None => return Ok(Flow::Continue),
            };

            let info = await!(cratesio::crate_info(&client, &crate_name));
            let info = match info {
                Ok(info) => info,
                // TODO: Use proper error types
                Err(ref err) if err.status() == Some(NotFound) => {
                    ctx.reply(&format!("Crate '{}' does not exist.", crate_name));
                    return Ok(Flow::Break)
                },
                Err(err) => {
                    eprintln!("Error getting crate info for '{}': {:?}", crate_name, err);
                    ctx.reply(&format!("Failed to get crate info for {}", crate_name));
                    return Ok(Flow::Break)
                }
                _ => unimplemented!(),
            };

            let krate = info.krate();
            let output = format!(
                "{name} ({version}) - {description} -> https://crates.io/crates/{urlname} [https://docs.rs/crate/{urlname}]",
                name = krate.name(),
                version = krate.max_version(),
                description = krate.description().split_whitespace().join(" "),
                urlname = utf8_percent_encode(&krate.name(), PATH_SEGMENT_ENCODE_SET).collect::<String>()
            );

            ctx.reply(&output);

            Ok(Flow::Break)
        })
    })
}
