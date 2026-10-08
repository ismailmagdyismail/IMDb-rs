pub enum RecordViewerMode {
    AllRecords,
    NextRecord,
    // RecordByKey,
    ResetCursor,
}

impl RecordViewerMode {
    pub fn parse(mode: &str) -> Option<Self> {
        match mode {
            "all" => Some(RecordViewerMode::AllRecords),
            "next" => Some(RecordViewerMode::NextRecord),
            // "record_by_key" => Some(RecordViewerMode::RecordByKey),
            "reset_cursor" => Some(RecordViewerMode::ResetCursor),
            _ => None,
        }
    }
}
