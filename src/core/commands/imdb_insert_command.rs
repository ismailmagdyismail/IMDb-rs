use crate::core::{
    imdb::Imdb, imdb_errors::error_message_formatter, record::imdb_record::ImdbRecord,
};

pub struct ImdbInsertCommand<'a> {
    imdb: &'a mut Imdb,
}

impl<'a> ImdbInsertCommand<'a> {
    pub fn new(imdb: &'a mut Imdb) -> ImdbInsertCommand<'a> {
        ImdbInsertCommand { imdb }
    }

    pub fn execute_command(&mut self, insert_args: ImdbInsertCommandArgs) {
        self.imdb
            .kv_store
            .insert(insert_args.record.key, insert_args.record.value);
    }
}

pub struct ImdbInsertCommandArgs {
    record: ImdbRecord,
}

impl ImdbInsertCommandArgs {
    pub fn parse(command_args: Vec<&str>) -> Result<ImdbInsertCommandArgs, String> {
        if command_args.len() != 2 {
            let error_message = format!(
                "[Imdb Insert Command Invalid args, expected 2, found {}]",
                command_args.len()
            );
            return Result::Err(error_message_formatter(error_message));
        }
        let key = command_args[0].to_owned();
        let value = command_args[1].to_owned();
        let record = ImdbRecord { key, value };
        Ok(ImdbInsertCommandArgs { record })
    }
}

#[cfg(test)]
mod test {
    use crate::core::commands::imdb_insert_command::ImdbInsertCommandArgs;

    #[test]
    fn test_command_args_parsing() {
        let args = vec!["key", "value"];
        let args = ImdbInsertCommandArgs::parse(args).unwrap();
        assert_eq!(args.record.key, "key");
        assert_eq!(args.record.value, "value");
    }
}
