use std::collections::HashMap;

use crate::core::{imdb_commands::ImdbCommand, imdb_insert_command::ImdbInsertCommand};

pub struct Imdb {
    pub kv_store: HashMap<String, String>,
}

impl Imdb {
    pub fn new() -> Imdb {
        Imdb {
            kv_store: HashMap::new(),
        }
    }

    pub fn execute_command(&mut self, command: String) -> Result<(), String> {
        let (command, args) = ImdbCommand::parse(&command)?;
        match command {
            ImdbCommand::Insert => self.handle_insert_command(args)?,
            ImdbCommand::Get => (),
            ImdbCommand::Delete => (),
        }
        Ok(())
    }

    fn handle_insert_command(&mut self, args: Vec<&str>) -> Result<(), String> {
        let insert_command_args = ImdbInsertCommand::parse(args)?;
        let mut insert_command = ImdbInsertCommand::new(self);
        insert_command.execute_command(insert_command_args);
        Ok(())
    }
}
