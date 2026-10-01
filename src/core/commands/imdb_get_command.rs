use crate::core::{
    imdb::Imdb, operations::imdb_errors::error_message_formatter, record::imdb_record::ImdbRecord,
    storage::imdb_storage_operations_status::ImdbStorageError,
};

pub struct ImdbGetCommand<'a> {
    imdb: &'a mut Imdb,
}

impl<'a> ImdbGetCommand<'a> {
    pub fn new(imdb: &'a mut Imdb) -> ImdbGetCommand<'a> {
        ImdbGetCommand { imdb }
    }

    pub fn execute_command(
        &mut self,
        command_args: ImdbGetCommandArgs,
    ) -> Result<Option<ImdbRecord>, ImdbStorageError> {
        let record = self.imdb.storage.read_record(command_args.key.to_vec())?;
        Ok(record)
    }
}

pub struct ImdbGetCommandArgs<'a> {
    key: &'a [u8],
}

impl<'a> ImdbGetCommandArgs<'a> {
    pub fn parse(args: Vec<&'a [u8]>) -> Result<ImdbGetCommandArgs<'a>, String> {
        if args.len() != 1 {
            return Result::Err(error_message_formatter(
                "[Imdb Get Command invalid args]".to_string(),
            ));
        }
        let key = args[0];
        Ok(ImdbGetCommandArgs { key })
    }
}
