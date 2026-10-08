use crate::core::{
    messages::format_error_message, record_viewer_config::RecordViewerConfig,
    record_viewer_operation::RecordViewerOperation, view_mode::RecordViewerMode,
};

pub fn parse_cli_args(
    args: &[String],
) -> Result<(RecordViewerConfig, RecordViewerOperation), String> {
    if args.len() < 4 {
        return Result::Err(format_error_message("Invalid number of arguments"));
    }
    let record_viewer_config = RecordViewerConfig::parse(&args[1..3])?;
    let viewer_mode = RecordViewerMode::parse(&args[3]);
    if viewer_mode.is_none() {
        return Result::Err(format_error_message("Invalid viewer mode"));
    }
    let record_viewer_operation = RecordViewerOperation {
        viewer_mode: viewer_mode.unwrap(),
    };
    return Ok((record_viewer_config, record_viewer_operation));
}
