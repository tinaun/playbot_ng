use serenity;
use serenity::model::{
    channel::Message,
    id::{ChannelId, UserId},
};

use threadpool::ThreadPool;

use regex::Regex;
use std::rc::Rc;

type SendFn = fn(&ThreadPool, ChannelId, &str) -> serenity::Result<()>;

#[derive(Clone)]
pub struct Context<'a> {
    body: &'a str,
    is_directly_addressed: bool,
    send_fn: SendFn,
    source: UserId,
    source_nickname: &'a str,
    target: ChannelId,
    client: &'a Message,
    pool: &'a ThreadPool,
    current_nickname: Rc<String>,
}

impl<'a> Context<'a> {
    pub fn new(pool: &'a ThreadPool, message: &'a Message) -> Option<Self> {
        let mut body = &message.content[..];

        let id = serenity::CACHE.read().user.id;

        let current_nickname = Rc::new(serenity::CACHE.read().user.name.to_owned());

        let source_nickname = &message.author.name;

        let source = message.author.id;

        let target = message.channel_id;

        let is_directly_addressed = {
            if body.starts_with(current_nickname.as_str()) {
                let new_body = body[current_nickname.len()..].trim_left();
                let has_separator = new_body.starts_with(":") || new_body.starts_with(",");

                if has_separator {
                    body = new_body[1..].trim_left();
                }

                has_separator
            } else {
                message.mentions_user_id(id)
            }
        };

        let send_fn: SendFn = |_pool, channel_id, msg| { channel_id.say(msg).map(|_| ()) }; 

        Some(Self {
            client: message,
            pool,
            body,
            send_fn,
            source,
            source_nickname,
            target,
            is_directly_addressed,
            current_nickname
        })
    }

    pub fn body(&self) -> &'a str {
        self.body
    }

    /// Wether the message was aimed directetly at the bot,
    /// either via private message or by prefixing a channel message with
    /// the bot's name, followed by ',' or ':'.
    pub fn is_directly_addressed(&self) -> bool {
        self.is_directly_addressed
    }

    pub fn is_ctcp(&self) -> bool {
        false
    }

    pub fn reply<S: AsRef<str>>(&self, message: S) {
        let message = message.as_ref();
        eprintln!("Replying: {:?}", message);
        for line in message.lines() {
            if line.len() > 2000 {
                let _ = (self.send_fn)(self.pool, self.target, "<<<message too long for irc>>>");
                continue;
            }
            let _ = (self.send_fn)(self.pool, self.target, line);
        }
    }

    pub fn source(&self) -> UserId {
        self.source
    }

    pub fn source_nickname(&self) -> &'a str {
        self.source_nickname
    }

    pub fn current_nickname(&self) -> Rc<String> {
        self.current_nickname.clone()
    }

    pub fn inline_contexts<'b>(&'b self) -> impl Iterator<Item = Context<'a>> + 'b {
        lazy_static! {
            static ref INLINE_CMD: Regex = Regex::new(r"\{(.*?)}").unwrap();
        }

        let body = if self.is_directly_addressed() { "" } else { self.body };

        let contexts = INLINE_CMD
            .captures_iter(body)
            .flat_map(|caps| caps.get(1))
            .map(move |body| Context {
                body: body.as_str(),
                .. self.clone()
            });
        
        Box::new(contexts)
    }
}
