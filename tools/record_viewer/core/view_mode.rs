pub enum RecordViewerMode {
    AllRecords,
    NextRecord,
    RecordByKey,
}

impl RecordViewerMode {
    pub fn parse(mode: &str) -> Option<Self> {
        match mode {
            "all" => Some(RecordViewerMode::AllRecords),
            "next" => Some(RecordViewerMode::NextRecord),
            "record_by_key" => Some(RecordViewerMode::RecordByKey),
            _ => None,
        }
    }
}
