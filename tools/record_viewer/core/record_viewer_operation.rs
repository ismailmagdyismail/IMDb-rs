use crate::core::view_mode::RecordViewerMode;

pub struct RecordViewerOperation {
    pub viewer_mode: RecordViewerMode,
}

impl RecordViewerOperation {
    pub fn new(args: &Vec<String>) -> Result<Self, String> {
        let mode = RecordViewerMode::parse(&args[2]);
        if mode.is_none() {
            return Result::Err("error".to_string());
        }
        let record_viewer_args = RecordViewerOperation {
            viewer_mode: mode.unwrap(),
        };
        return Ok(record_viewer_args);
    }
}
