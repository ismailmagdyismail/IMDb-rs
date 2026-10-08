use crate::core::{messages::format_error_message, record_viewer_config::RecordViewerConfig};

pub fn parse_cli_args(args: &[String]) -> Result<RecordViewerConfig, String> {
    if args.len() != 3 {
        return Result::Err(format_error_message("Invalid number of arguments"));
    }
    let record_viewer_config = RecordViewerConfig::parse(&args[1..3])?;
    return Ok(record_viewer_config);
}
