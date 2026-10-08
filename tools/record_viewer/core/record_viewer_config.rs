use imdb::core::operations::imdb_format::ImdbFormat;

use crate::core::messages::format_error_message;

pub struct RecordViewerConfig {
    pub imdb_format: ImdbFormat,
    pub db_dir_path: String,
}

impl RecordViewerConfig {
    pub fn parse(args: &[String]) -> Result<Self, String> {
        if args.len() < 2 {
            return Result::Err(format_error_message("Invalid number of arguments").to_string());
        }
        let imdb_format = ImdbFormat::from_str(&args[1]);
        if imdb_format.is_none() {
            return Result::Err(format_error_message("Invalid imdb format").to_string());
        }
        let record_viewer_config_args = RecordViewerConfig {
            imdb_format: imdb_format.unwrap(),
            db_dir_path: args[0].clone(),
        };
        return Ok(record_viewer_config_args);
    }
}
