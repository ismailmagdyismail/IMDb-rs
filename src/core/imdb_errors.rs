use crate::core::imdb_messages::USAGE_MESSAGE;

pub fn error_message_formatter(error: String) -> String {
    let mut formatted_error_message = String::from("\n\n");
    formatted_error_message.push_str(&error);
    formatted_error_message.push_str("\n");
    formatted_error_message.push_str(USAGE_MESSAGE);
    formatted_error_message
}
