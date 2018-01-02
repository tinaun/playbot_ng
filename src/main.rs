extern crate serde;
extern crate failure;
extern crate irc;
extern crate reqwest;
extern crate url;
#[macro_use] extern crate serde_derive;

mod playground;
mod paste;
mod cratesio;
mod bot;

fn main() {
    loop {   
        if let Ok(e) = bot::run() {
            eprintln!("Disconnected because: {:?}", e);
        } else {
            eprintln!("Disconnected for an unknown reason");
        }
    }
}
