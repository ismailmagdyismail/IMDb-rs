pub const USAGE_MESSAGE: &'static str = "record-viewer [Imdb_DB_DIR_PATH] [inline_metadata|seperate_metadata] [all|next|record_with_key] [key (if record_with_key)]
    Imdb_DB_DIR_PATH: path to the directory containing the db files (data files, indexes, metadata files)
    inline_metadata: indicates file database format is in inline metadata format
    seperate_metadata: indicates file database format is in seperate metadata format
    all: view all records in the db
    next_record: view the next record in the db
    record_with_key: view the record with the specified key in the db
    key: the key of the record to view (only required if record_with_key is specified)";

pub fn format_error_message(error: &str) -> String {
    let mut formatted_error_message = String::from("\n");
    formatted_error_message.push_str(&format!("Error: {}\n", error));
    formatted_error_message.push_str(USAGE_MESSAGE);
    formatted_error_message
}
