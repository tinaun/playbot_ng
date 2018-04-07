use {Context, Flow};

pub fn handler(ctx: &Context, _args: &[&str]) -> Flow {
    display_help(ctx);
    Flow::Break
}

pub fn display_help(ctx: &Context) {
    ctx.reply("Usage help can be found here: https://github.com/panicbit/playbot_ng/tree/master/README.md");
}
