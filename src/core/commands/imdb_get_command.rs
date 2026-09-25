use crate::core::{imdb::Imdb, imdb_errors::error_message_formatter};

pub struct ImdbGetCommand<'a> {
    imdb: &'a Imdb,
}

impl<'a> ImdbGetCommand<'a> {
    pub fn new(imdb: &'a Imdb) -> ImdbGetCommand<'a> {
        ImdbGetCommand { imdb }
    }

    pub fn execute_command(&self, command_args: ImdbGetCommandArgs) -> Option<&'a Vec<u8>> {
        let value = self.imdb.kv_store.get(command_args.key);
        return value;
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
