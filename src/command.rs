pub struct Command<'msg> {
    name: &'msg str,
    args: Vec<&'msg str>,
}

impl<'msg> Command<'msg> {
    pub fn parse(prefix: &str, msg: &'msg str) -> Option<Self> {
        let msg = msg.trim();
        let mut args = msg.split_whitespace();
        let name = args.next()
            .filter(|name| name.starts_with(prefix))
            .map(|name| &name[prefix.len()..])?;

        Some(Command {
            name: name,
            args: args.collect(),
        })
    }

    pub fn name(&self) -> &'msg str {
        self.name
    }
    
    pub fn args(&self) -> &[&'msg str] {
        &self.args
    }
}
