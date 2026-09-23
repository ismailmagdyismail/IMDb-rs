use crate::core::imdb_errors::error_message_formatter;

#[derive(Debug)]
pub enum ImdbCommand {
    Get,
    Insert,
    Delete,
}

impl ImdbCommand {
    pub fn parse(command: &String) -> Result<ImdbCommand, String> {
        let command_entries: Vec<&str> = command.split(' ').collect();
        let command_name = command_entries[0];
        let normalized_command_name = command_name.to_lowercase();
        let normalized_command_name = normalized_command_name.trim();
        match normalized_command_name {
            "get" => Ok(ImdbCommand::Get),
            "insert" => Ok(ImdbCommand::Insert),
            "delete" => Ok(ImdbCommand::Delete),
            _ => {
                return Result::Err(error_message_formatter(
                    "[Imdb Invalid Command name]".to_string(),
                ));
            }
        }
    }
}
