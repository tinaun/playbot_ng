use irc::client::prelude::{Message, IrcClient};
use std::collections::HashMap;
use super::{Context, Flow, Command};

pub struct CommandRegistry {
    command_prefix: String,
    named_handlers: HashMap<String, Box<FnMut(&Context, &[&str]) -> Flow>>,
    fallback_handlers: Vec<Box<FnMut(&Context) -> Flow>>,
}

impl CommandRegistry {
    pub fn new(command_prefix: impl Into<String>) -> Self {
        Self {
            command_prefix: command_prefix.into(),
            named_handlers: HashMap::new(),
            fallback_handlers: Vec::new(),
        }
    }

    pub fn set_named_handler(
        &mut self,
        name: impl Into<String>,
        handler: impl Fn(&Context, &[&str]) -> Flow + 'static,
    ) {
        self.named_handlers.insert(name.into(), Box::new(handler));  
    }

    pub fn add_fallback_handler(
        &mut self,
        handler: impl Fn(&Context) -> Flow + 'static,
    ) {
        self.fallback_handlers.push(Box::new(handler));
    }

    pub fn handle_message(&mut self, client: &IrcClient, message: &Message) {
        let context = match Context::new(&client, &message) {
            Some(context) => context,
            None => return,
        };

        if context.is_ctcp() {
            return;
        }

        if let Some(command) = Command::parse(&self.command_prefix, context.body()) {
            if let Some(handler) = self.named_handlers.get_mut(command.name()) {
                if handler(&context, command.args()) == Flow::Break {
                    return;
                }
            }
        }

        for handler in &mut self.fallback_handlers {
            if handler(&context) == Flow::Break {
                return;
            }
        }
    }
}
