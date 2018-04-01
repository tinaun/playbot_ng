use irc;
use irc::client::prelude::*;
use regex::Regex;
use shared_str::SharedStr;

type SendFn = fn(&IrcClient, &str, &str) -> irc::error::Result<()>;

#[derive(Clone)]
pub struct Context {
    body: SharedStr,
    is_directly_addressed: bool,
    is_ctcp: bool,
    send_fn: SendFn,
    source: SharedStr,
    source_nickname: SharedStr,
    target: SharedStr,
    client: IrcClient,
    current_nickname: SharedStr,
}

impl Context {
    pub fn new(client: &IrcClient, message: Message) -> Option<Self> {
        let client = client.clone();

        let source_nickname = message.source_nickname().map(SharedStr::from)?;

        let target = match message.response_target() {
            Some(target) => SharedStr::from(target),
            None => {
                eprintln!("Unknown response target");
                return None;
            }
        };

        let mut body = match message.command {
            Command::PRIVMSG(_, body) => SharedStr::from(body).trim(),
            _ => return None,
        };

        let source = message.prefix.map(SharedStr::from)?;

        let is_ctcp = body.len() >= 2 && body.chars().next() == Some('\x01')
            && body.chars().last() == Some('\x01');

        if is_ctcp {
            body = body.slice(1..body.len() - 1);
        }

        let is_directly_addressed = {
            let current_nickname = client.current_nickname();

            if body.starts_with(current_nickname) {
                let new_body = body.slice(current_nickname.len()..).trim_left();

                if new_body.starts_with(":") || new_body.starts_with(",") {
                    body = new_body.slice(1..).trim_left();
                    true
                } else {
                    false
                }
            } else {
                !target.as_str().is_channel_name()
            }
        };

        let send_fn: SendFn = match target.as_str().is_channel_name() {
            true => |client, target, message| client.send_notice(target, message),
            false => |client, target, message| client.send_privmsg(target, message),
        };

        let current_nickname = SharedStr::from(client.current_nickname());

        Some(Self {
            client,
            body,
            send_fn,
            source,
            source_nickname,
            target,
            is_directly_addressed,
            is_ctcp,
            current_nickname
        })
    }

    pub fn body(&self) -> &SharedStr {
        &self.body
    }

    /// Wether the message was aimed directetly at the bot,
    /// either via private message or by prefixing a channel message with
    /// the bot's name, followed by ',' or ':'.
    pub fn is_directly_addressed(&self) -> bool {
        self.is_directly_addressed
    }

    pub fn is_ctcp(&self) -> bool {
        self.is_ctcp
    }

    pub fn reply(&self, message: &str) {
        (self.send_fn)(&self.client, &self.target, &message);
    }

    pub fn source(&self) -> &SharedStr {
        &self.source
    }

    pub fn source_nickname(&self) -> &SharedStr {
        &self.source_nickname
    }

    pub fn current_nickname(&self) -> &SharedStr {
        &self.current_nickname
    }

    pub fn inline_contexts<'a>(&'a self) -> impl Iterator<Item = Context> + 'a {
        lazy_static! {
            static ref INLINE_CMD: Regex = Regex::new(r"\{(.*?)}").unwrap();
        }

        let body = if self.is_directly_addressed() { "" } else { &self.body };

        let contexts = INLINE_CMD
            .captures_iter(body)
            .flat_map(|caps| caps.get(1))
            .map(move |mat| Context {
                body: self.body.from_str(mat.as_str()),
                .. self.clone()
            });
        
        Box::new(contexts)
    }
}
