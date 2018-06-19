use std::path::Path;
use std::fs;

use toml;
use failure::{Error, err_msg};

pub struct Config {
    token: String,
}

impl Config {
    pub fn load<T: AsRef<Path>>(path: T) -> Result<Self, Error> {
        let file = fs::read_to_string(path)?;
        let file: toml::Value = toml::from_str(&file)?;
        let token = match file.get("token") {
            Some(tok) => tok.to_string(),
            None => return Err(err_msg("field `token` not found!")),
        };       

        Ok(Config {
            token
        })
    }

    pub fn token(&self) -> &str {
        &self.token
    }
}