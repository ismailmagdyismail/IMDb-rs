use crate::core::{
    imdb_config::ImdbConfig, imdb_errors::error_message_formatter, imdb_opeartion::ImdbOperation,
};

pub struct ImdbCliArgsParser {}

impl ImdbCliArgsParser {
    pub fn parse(args: Vec<String>) -> Result<(ImdbConfig, ImdbOperation), String> {
        if args.len() < 3 {
            return Result::Err(error_message_formatter("[Imdb Invalid Args]".to_string()));
        }

        let db_path = args[1].clone(); // TODO: remove clone , take ownership | borrow
        let imdb_config = ImdbConfig::new(db_path)?;
        let imdb_operation = ImdbOperation::from(&args[2])?;
        Ok((imdb_config, imdb_operation))
    }
}
