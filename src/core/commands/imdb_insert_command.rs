use crate::core::{
    imdb::Imdb, operations::imdb_errors::error_message_formatter, record::imdb_record::ImdbRecord,
    storage::imdb_storage_operations_status::ImdbStorageError,
};

pub struct ImdbInsertCommand<'a> {
    imdb: &'a mut Imdb,
}

impl<'a> ImdbInsertCommand<'a> {
    pub fn new(imdb: &'a mut Imdb) -> ImdbInsertCommand<'a> {
        ImdbInsertCommand { imdb }
    }

    pub fn execute_command(
        &mut self,
        insert_args: ImdbInsertCommandArgs,
    ) -> Result<(), ImdbStorageError> {
        self.imdb.storage.write_record(insert_args.record)?;
        Ok(())
    }
}

pub struct ImdbInsertCommandArgs {
    record: ImdbRecord,
}

impl ImdbInsertCommandArgs {
    pub fn parse(command_args: Vec<&[u8]>) -> Result<ImdbInsertCommandArgs, String> {
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
        let args = vec!["key".as_bytes(), "value".as_bytes()];
        let parsed_args = ImdbInsertCommandArgs::parse(args).unwrap();
        assert_eq!(parsed_args.record.key, "key".as_bytes());
        assert_eq!(parsed_args.record.value, "value".as_bytes());
    }
}
