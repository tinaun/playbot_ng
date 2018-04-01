use shared_str::SharedStr;

pub struct Command {
    name: SharedStr,
    args: Vec<SharedStr>,
}

impl Command {
    pub fn parse(prefix: impl Into<SharedStr>, msg: impl Into<SharedStr>) -> Option<Self> {
        let prefix = prefix.into();
        let msg = msg.into().trim();
        let mut args = msg.split_whitespace();
        let name = args.next()
            .filter(|name| name.starts_with(&*prefix))
            .map(|name| name.slice(prefix.len()..))?;

        Some(Command {
            name: name,
            args: args.collect(),
        })
    }

    pub fn name(&self) -> &SharedStr {
        &self.name
    }
    
    pub fn args(&self) -> &[SharedStr] {
        &self.args
    }
}
