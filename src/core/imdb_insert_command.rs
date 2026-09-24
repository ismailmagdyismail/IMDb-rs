use crate::core::{imdb::Imdb, imdb_errors::error_message_formatter};

pub struct ImdbInsertCommand<'a> {
    imdb: &'a mut Imdb,
}

pub struct ImdbInsertCommandArgs {
    key: String,
    value: String,
}

impl<'a> ImdbInsertCommand<'a> {
    pub fn new(imdb: &'a mut Imdb) -> ImdbInsertCommand<'a> {
        ImdbInsertCommand { imdb }
    }

    pub fn execute_command(&mut self, insert_args: ImdbInsertCommandArgs) {
        self.imdb
            .kv_store
            .insert(insert_args.key, insert_args.value);
    }
}

impl<'a> ImdbInsertCommand<'a> {
    pub fn parse(command_args: Vec<&'a str>) -> Result<ImdbInsertCommandArgs, String> {
        if command_args.len() != 2 {
            let error_message = format!(
                "[Imdb Insert Command Invalid args, expected 2, found {}]",
                command_args.len()
            );
            return Result::Err(error_message_formatter(error_message));
        }
        let key = command_args[0].to_owned();
        let value = command_args[1].to_owned();
        Ok(ImdbInsertCommandArgs { key, value })
    }
}

#[cfg(test)]
mod test {
    use crate::core::imdb_insert_command::ImdbInsertCommand;

    #[test]
    fn test_command_args_parsing() {
        let args = vec!["key", "value"];
        let args = ImdbInsertCommand::parse(args).unwrap();
        assert_eq!(args.key, "key");
        assert_eq!(args.value, "value");
    }
}
