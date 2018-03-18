use super::{Context, Flow, Module};
use regex::Regex;
use itertools::Itertools;
use std::iter::once;

pub struct Egg {
    script: Vec<(Regex, fn(&str) -> String)>,
}

fn re(re: &str) -> Regex {
    let re = once("(?i)")
        .chain(re.split_whitespace())
        .chain(once(""))
        .join(r"\s*");
    Regex::new(&re).unwrap()
}

impl Egg {
    pub fn new() -> Self {
        Self {
            script: vec![
                (
                    re(r"Open the pod bay doors? ,? (?P<nick>[[:word:]]+) [.!]?"),
                    |name| format!("I'm sorry {}, I'm afraid I can't do that.", name),
                ),
                (
                    re(r"(What'?s|What is|Wats) the problem \??"),
                    |_| format!("I think you know what the problem is just as well as I do."),
                ),
                (
                    re(r"What are you talking about ,? (?P<nick>[[:word:]]+) \??"),
                    |_| format!("This mission is too important for me to allow you to jeopardize it.")
                ),
                (
                    re(r"I (don't|dont) know what you are talking about ,? (?P<nick>[[:word:]]+) [.?!]?"),
                    |name| {
                        let other = match name.to_lowercase().as_str() {
                            "panicbit" => "Rantanen",
                            "rantanen" => "panicbit",
                            "graydon" => "steveklabnik",
                            _ => "Graydon",
                        };
                        format!("I know that you and {} were planning to disconnect me and I'm afraid that's something I cannot allow to happen", other)
                    }
                ),
            ],
        }
    }
}

impl Module for Egg {
    fn run(&mut self, ctx: Context) -> Flow {
        for dialog in &self.script {
            if let Some(caps) = dialog.0.captures(ctx.body) {
                if let Some(nick) = caps.name("nick") {
                    if nick.as_str() != ctx.current_nickname() {
                        return Flow::Break;
                    }
                }

                ctx.reply(&(dialog.1)(ctx.source_nickname()));

                return Flow::Break;
            }
        }

        // if let Some(name) = self.re1.captures(ctx.body).and_then(|c| c.get(1)) {
        //     if name.as_str() == ctx.current_nickname() {
        //         ctx.reply(&format!(
        //             "I'm sorry {}, I'm afraid I can't do that.",
        //             ctx.source_nickname()
        //         ));
        //         return Flow::Break;
        //     }
        // }

        // if self.re2.is_match(ctx.body) {
        //     ctx.reply(&format!(
        //         "I think you know what the problem is just as well as I do."
        //     ));
        //     return Flow::Break;
        // }

        Flow::Continue
    }
}
