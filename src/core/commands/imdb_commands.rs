use crate::core::operations::imdb_errors::error_message_formatter;

#[derive(Debug, PartialEq)]
pub enum ImdbCommand {
    Get,
    Insert,
    Delete,
}

impl ImdbCommand {
    pub fn parse(command: &Vec<u8>) -> Result<(ImdbCommand, &[u8], Vec<&[u8]>), String> {
        let mut command_entries: Vec<&[u8]> = command.split(|byte| *byte == b' ').collect();
        if command_entries.is_empty() {
            return Result::Err(error_message_formatter(
                "[Imdb Ivalid Command -- empty command]".to_string(),
            ));
        }
        let args = command_entries.split_off(1);
        let command_name = command_entries[0];
        let normalized_command_name = command_name.trim_ascii().to_ascii_lowercase();
        let normalized_command_name = normalized_command_name.as_slice();
        match normalized_command_name {
            b"get" => Ok((ImdbCommand::Get, command_name, args)),
            b"insert" => Ok((ImdbCommand::Insert, command_name, args)),
            b"delete" => Ok((ImdbCommand::Delete, command_name, args)),
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
    use crate::core::commands::imdb_commands::ImdbCommand;

    #[test]
    fn test_command_type() {
        let (get, _, _) = ImdbCommand::parse(&"get".to_string().into_bytes()).unwrap();
        let (insert, _, _) = ImdbCommand::parse(&"insert".to_string().into_bytes()).unwrap();
        let (delete, _, _) = ImdbCommand::parse(&"delete".to_string().into_bytes()).unwrap();

        assert_eq!(get, ImdbCommand::Get);
        assert_eq!(insert, ImdbCommand::Insert);
        assert_eq!(delete, ImdbCommand::Delete);
    }

    #[test]
    fn test_case_insensitve_command() {
        let (get, _, _) = ImdbCommand::parse(&"Get".to_string().into_bytes()).unwrap();
        let (insert, _, _) = ImdbCommand::parse(&"INSERT".to_string().into_bytes()).unwrap();
        let (delete, _, _) = ImdbCommand::parse(&"DelETe".to_string().into_bytes()).unwrap();

        assert_eq!(get, ImdbCommand::Get);
        assert_eq!(insert, ImdbCommand::Insert);
        assert_eq!(delete, ImdbCommand::Delete);
    }

    #[test]
    fn test_args_split() {
        let command = "Get".to_string().into_bytes();
        let (get, _, args) = ImdbCommand::parse(&command).unwrap();
        assert_eq!(get, ImdbCommand::Get);
        assert_eq!(args.len(), 0);

        let command = "Insert key value".to_string().into_bytes();
        let (get, _, args) = ImdbCommand::parse(&command).unwrap();
        assert_eq!(get, ImdbCommand::Insert);
        assert_eq!(args.len(), 2);
    }
}
