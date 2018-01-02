use reqwest;
use url::percent_encoding::{utf8_percent_encode, PATH_SEGMENT_ENCODE_SET};
use failure::Error;

/*
pub struct CratesIo;

pub fn init() -> CratesIo {
    CratesIo
}

impl Module for CratesIo {
    fn recognizes(&self, command: &str) -> bool {
        command.starts_with("crate ")
    }
    fn execute(&self, command: &str, config: &super::Config, irc: Arc<Irc>, channel: Arc<Channel>) -> Result<(), Box<error::Error>> {
        let tag = "crate ";
        if !self.recognizes(command) {
            return Err("unknown command".into())
        }

        let command = command[tag.len()..].trim();
        let krate = try!(crate_info(command)).krate;
        let output = format!(
            "{name} ({version}) - {description} -> https://crates.io/crates/{urlname} [https://docs.rs/crate/{urlname}]",
            name = krate.name,
            version = krate.max_version,
            description = krate.description,
            urlname = utf8_percent_encode(&krate.name, PATH_SEGMENT_ENCODE_SET).collect::<String>()
        );

        irc.notice(channel.name(), &output);

        Ok(())
    }
}
*/

#[derive(Deserialize,Debug,Clone,PartialEq,Eq)]
pub struct Info {
    #[serde(rename = "crate")]
    krate: Crate,
}

#[derive(Deserialize,Debug,Clone,PartialEq,Eq)]
pub struct Crate {
    id: String,
    name: String,
    description: String,
    max_version: String,
}

impl Info {
    pub fn krate(&self) -> &Crate {
        &self.krate
    }
}

impl Crate {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn max_version(&self) -> &str {
        &self.max_version
    }
}

pub fn crate_info(name: &str) -> Result<Info, Error> {
    let url = format!(
        "https://crates.io/api/v1/crates/{}",
        utf8_percent_encode(name, PATH_SEGMENT_ENCODE_SET).collect::<String>()
    );
    let info = reqwest::get(&url)?
        .error_for_status()?
        .json()?;

    Ok(info)
}
