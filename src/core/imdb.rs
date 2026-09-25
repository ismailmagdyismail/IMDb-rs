use std::collections::HashMap;

use crate::core::{
    commands::{
        imdb_commands::ImdbCommand,
        imdb_get_command::{ImdbGetCommand, ImdbGetCommandArgs},
        imdb_insert_command::{ImdbInsertCommand, ImdbInsertCommandArgs},
    },
    record::imdb_record::{ImdbRecordKey, ImdbRecordValue},
};

pub struct Imdb {
    pub kv_store: HashMap<ImdbRecordKey, ImdbRecordValue>,
}

impl Imdb {
    pub fn new() -> Imdb {
        Imdb {
            kv_store: HashMap::new(),
        }
    }

    pub fn execute_command(&mut self, command: Vec<u8>) -> Result<Option<&Vec<u8>>, String> {
        let (command, _, args) = ImdbCommand::parse(&command)?;
        match command {
            ImdbCommand::Insert => {
                self.handle_insert_command(args)?;
                return Ok(None);
            }
            ImdbCommand::Get => {
                let value = self.handle_get_command(args)?;
                return Ok(value);
            }
            ImdbCommand::Delete => {
                return Ok(None);
            }
        }
    }

    fn handle_insert_command(&mut self, args: Vec<&[u8]>) -> Result<(), String> {
        let insert_command_args = ImdbInsertCommandArgs::parse(args)?;
        let mut insert_command = ImdbInsertCommand::new(self);
        insert_command.execute_command(insert_command_args);
        Ok(())
    }

    fn handle_get_command(&self, args: Vec<&[u8]>) -> Result<Option<&Vec<u8>>, String> {
        let insert_command_args = ImdbGetCommandArgs::parse(args)?;
        let get_command = ImdbGetCommand::new(self);
        let val = get_command.execute_command(insert_command_args);
        return Ok(val);
    }
}
