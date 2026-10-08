pub const USAGE_MESSAGE: &'static str = "./record-viewer [Imdb_DB_DIR_PATH] [inline_metadata|seperate_metadata]
    - Imdb_DB_DIR_PATH: path to the directory containing the db files (data files, indexes, metadata files)

    - Database Format:
        * inline_metadata: indicates file database format is in inline metadata format
        * seperate_metadata: indicates file database format is in seperate metadata format
    
    >> Commands: [all|next|record_with_key] [key (if record_with_key)]
    - View Options:
        * all: view all records in the db
        * next_record: view the next record in the db
        * reset_cursor: reset viewer cursor to the first record, to be able to iterate over them again.
    ";

pub fn format_error_message(error: &str) -> String {
    let mut formatted_error_message = String::from("\n");
    formatted_error_message.push_str(&format!("Error: {}\n", error));
    formatted_error_message.push_str(USAGE_MESSAGE);
    formatted_error_message
}
