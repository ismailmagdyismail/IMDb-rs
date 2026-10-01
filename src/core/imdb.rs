use std::path::Path;

use crate::core::{
    commands::{
        imdb_commands::ImdbCommand,
        imdb_get_command::{ImdbGetCommand, ImdbGetCommandArgs},
        imdb_insert_command::{ImdbInsertCommand, ImdbInsertCommandArgs},
    },
    operations::imdb_config::ImdbConfig,
    record::imdb_record::ImdbRecord,
    storage::imdb_storage_engine::ImdbStorageEngine,
};

pub struct Imdb {
    pub config: ImdbConfig,
    pub storage: ImdbStorageEngine,
}

impl Imdb {
    pub fn new(config: ImdbConfig) -> Result<Imdb, String> {
        let db_dir_path = Path::new(&config.db_path);
        let storage = ImdbStorageEngine::new(db_dir_path)?;

        let db = Imdb { config, storage };
        Ok(db)
    }

    pub fn execute_command(&mut self, command: Vec<u8>) -> Result<Option<Vec<u8>>, String> {
        let (command, _, args) = ImdbCommand::parse(&command)?;
        match command {
            ImdbCommand::Insert => {
                self.handle_insert_command(args)?;
                return Ok(None);
            }
            ImdbCommand::Get => {
                let value = self.handle_get_command(args)?;
                if let Some(record) = value {
                    Ok(Some(record.value))
                } else {
                    Ok(None)
                }
            }
            ImdbCommand::Delete => {
                return Ok(None);
            }
        }
    }

    fn handle_insert_command(&mut self, args: Vec<&[u8]>) -> Result<(), String> {
        let insert_command_args = ImdbInsertCommandArgs::parse(args)?;
        let mut insert_command = ImdbInsertCommand::new(self);
        insert_command.execute_command(insert_command_args)?;
        Ok(())
    }

    fn handle_get_command(&mut self, args: Vec<&[u8]>) -> Result<Option<ImdbRecord>, String> {
        let insert_command_args = ImdbGetCommandArgs::parse(args)?;
        let mut get_command = ImdbGetCommand::new(self);
        let val = get_command.execute_command(insert_command_args)?;
        return Ok(val);
    }
}
