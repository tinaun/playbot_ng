use irc::client::prelude::{Message, IrcClient};
use std::collections::HashMap;
use super::{Context, Flow, Command, FlowFuture};
use std::iter;
use shared_str::SharedStr;
use futures::prelude::*;
use std::sync::Arc;

pub struct CommandRegistry {
    command_prefix: SharedStr,
    named_handlers: HashMap<SharedStr, Arc<Fn(Context, &[SharedStr]) -> FlowFuture>>,
    fallback_handlers: Vec<Arc<Fn(Context) -> FlowFuture>>,
}

impl CommandRegistry {
    pub fn new(command_prefix: impl Into<SharedStr>) -> Self {
        Self {
            command_prefix: command_prefix.into(),
            named_handlers: HashMap::new(),
            fallback_handlers: Vec::new(),
        }
    }

    pub fn set_named_handler(
        &mut self,
        name: impl Into<SharedStr>,
        handler: Arc<Fn(Context, &[SharedStr]) -> FlowFuture + 'static>,
    ) {
        self.named_handlers.insert(name.into(), handler);  
    }

    pub fn add_fallback_handler(
        &mut self,
        handler: Arc<Fn(Context) -> FlowFuture + 'static>,
    ) {
        self.fallback_handlers.push(handler);
    }

    pub fn handle_message<E>(&mut self, client: &IrcClient, message: Message) -> impl Future<Item = (), Error = E> {
        let client: IrcClient = client.clone();
        let command_prefix = self.command_prefix.clone();
        let named_handlers = self.named_handlers.clone();
        let fallback_handlers = self.fallback_handlers.clone();

        async_block! {
            let client = client;
            let context = match Context::new(&client, message) {
                Some(context) => context,
                None => return Ok(()),
            };

            if context.is_ctcp() {
                return Ok(());
            }

            // Handle the main context first
            if let Some(command) = Command::parse(&command_prefix, context.body()) {
                if let Some(handler) = named_handlers.get(command.name()).cloned() {
                    if await!(handler(context.clone(), command.args())) == Ok(Flow::Break) {
                        return Ok(());
                    }
                }
            }

            // Then handle ALL inline contexts before deciding flow
            let contexts = iter::once(context.clone())
                .chain(context.inline_contexts())
                .take(3)
                .collect::<Vec<_>>();
            let mut any_inline_command_succeded = false;
            for context in contexts {
                if let Some(command) = Command::parse(&command_prefix, context.body()) {
                    if let Some(handler) = named_handlers.get(command.name()).cloned() {
                        if await!(handler(context, command.args())) == Ok(Flow::Break) {
                            any_inline_command_succeded = true;
                        }
                    }
                }
            }

            if any_inline_command_succeded {
                return Ok(());
            }

            for handler in fallback_handlers {
                if await!(handler(context.clone())) == Ok(Flow::Break) {
                    return Ok(());
                }
            }

            Ok(())
        }
    }
}
