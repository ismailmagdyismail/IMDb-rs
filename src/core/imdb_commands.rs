use crate::core::imdb_errors::error_message_formatter;

#[derive(Debug, PartialEq)]
pub enum ImdbCommand {
    Get,
    Insert,
    Delete,
}

impl ImdbCommand {
    pub fn parse(command: &String) -> Result<(ImdbCommand, Vec<&str>), String> {
        let mut command_entries: Vec<&str> = command.split(' ').collect();
        if command_entries.is_empty() {
            return Result::Err(error_message_formatter(
                "[Imdb Ivalid Command -- empty command]".to_string(),
            ));
        }
        let args = command_entries.split_off(1);
        let command_name = command_entries[0];
        let normalized_command_name = command_name.to_lowercase();
        let normalized_command_name = normalized_command_name.trim();
        match normalized_command_name {
            "get" => Ok((ImdbCommand::Get, args)),
            "insert" => Ok((ImdbCommand::Insert, args)),
            "delete" => Ok((ImdbCommand::Delete, args)),
            _ => {
                return Result::Err(error_message_formatter(
                    "[Imdb Invalid Command name]".to_string(),
                ));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::core::imdb_commands::ImdbCommand;

    #[test]
    fn test_command_type() {
        let (get, _) = ImdbCommand::parse(&"get".to_string()).unwrap();
        let (insert, _) = ImdbCommand::parse(&"insert".to_string()).unwrap();
        let (delete, _) = ImdbCommand::parse(&"delete".to_string()).unwrap();

        assert_eq!(get, ImdbCommand::Get);
        assert_eq!(insert, ImdbCommand::Insert);
        assert_eq!(delete, ImdbCommand::Delete);
    }

    #[test]
    fn test_case_insensitve_command() {
        let (get, _) = ImdbCommand::parse(&"Get".to_string()).unwrap();
        let (insert, _) = ImdbCommand::parse(&"INSERT".to_string()).unwrap();
        let (delete, _) = ImdbCommand::parse(&"DelETe".to_string()).unwrap();

        assert_eq!(get, ImdbCommand::Get);
        assert_eq!(insert, ImdbCommand::Insert);
        assert_eq!(delete, ImdbCommand::Delete);
    }

    #[test]
    fn test_args_split() {
        let command = "Get".to_string();
        let (get, args) = ImdbCommand::parse(&command).unwrap();
        assert_eq!(get, ImdbCommand::Get);
        assert_eq!(args.len(), 0);

        let command = "Insert key value".to_string();
        let (get, args) = ImdbCommand::parse(&command).unwrap();
        assert_eq!(get, ImdbCommand::Insert);
        assert_eq!(args.len(), 2);
    }
}
