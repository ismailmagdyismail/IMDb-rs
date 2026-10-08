
pub enum RecordViewerMode {
    AllRecords,
    NextRecord,
    RecordByKey,
}

impl RecordViewerMode {
    pub fn parse(mode: &String) -> Option<Self> {
        match mode.as_str() {
            "all" => Some(RecordViewerMode::AllRecords),
            "next" => Some(RecordViewerMode::NextRecord),
            "record_by_key" => Some(RecordViewerMode::RecordByKey),
            _ => None,
        }
    }
}