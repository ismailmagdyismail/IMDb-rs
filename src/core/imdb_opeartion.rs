use crate::core::imdb_errors::error_message_formatter;

#[derive(Debug)]
pub enum ImdbOperation {
    Repl,
    Deamon,
}

impl ImdbOperation {
    pub fn from(operation_arg: &String) -> Result<ImdbOperation, String> {
        let normalized_imdb_operation_arg = operation_arg.to_lowercase();
        let normalized_imdb_operation_arg = normalized_imdb_operation_arg.trim();
        let imdb_operation = match normalized_imdb_operation_arg {
            "deamon" => ImdbOperation::Deamon,
            "repl" => ImdbOperation::Repl,
            _ => {
                return Result::Err(error_message_formatter(
                    "[Imdb Invalid operation]".to_string(),
                ));
            }
        };

        Ok(imdb_operation)
    }
}
