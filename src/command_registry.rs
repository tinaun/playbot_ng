use serenity::framework::Framework;
use serenity::model::channel::Message;
use serenity::prelude::Context as SerenityContext;

use threadpool::ThreadPool;

use std::collections::HashMap;
use super::{Context, Flow, Command};
use std::iter;

pub struct CommandRegistry {
    command_prefix: String,
    named_handlers: HashMap<String, Box<FnMut(&Context, &[&str]) -> Flow + Send>>,
    fallback_handlers: Vec<Box<FnMut(&Context) -> Flow + Send>>,
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
        handler: impl Fn(&Context, &[&str]) -> Flow + Send + 'static,
    ) {
        self.named_handlers.insert(name.into(), Box::new(handler));  
    }

    pub fn add_fallback_handler(
        &mut self,
        handler: impl Fn(&Context) -> Flow + Send + 'static,
    ) {
        self.fallback_handlers.push(Box::new(handler));
    }

    pub fn handle_message(&mut self, pool: &ThreadPool, message: &Message) {
        let context = match Context::new(&pool, &message) {
            Some(context) => context,
            None => return,
        };

        if context.is_ctcp() {
            return;
        }

        // Handle the main context first
        if let Some(command) = Command::parse(&self.command_prefix, context.body()) {
            if let Some(handler) = self.named_handlers.get_mut(command.name()) {
                if handler(&context, command.args()) == Flow::Break {
                    return;
                }
            }
        }

        // Then handle ALL inline contexts before deciding flow
        let contexts = iter::once(context.clone()).chain(context.inline_contexts());
        let mut any_inline_command_succeded = false;
        for context in contexts.take(3) {
            if let Some(command) = Command::parse(&self.command_prefix, context.body()) {
                if let Some(handler) = self.named_handlers.get_mut(command.name()) {
                    if handler(&context, command.args()) == Flow::Break {
                        any_inline_command_succeded = true;
                    }
                }
            }
        }

        if any_inline_command_succeded {
            return;
        }

        for handler in &mut self.fallback_handlers {
            if handler(&context) == Flow::Break {
                return;
            }
        }
    }
}


impl Framework for CommandRegistry {
    fn dispatch(&mut self, _: SerenityContext, m: Message, t: &ThreadPool) {
        self.handle_message(t, &m);
    }
}