use std::fmt::Display;

use imdb::core::storage::pager::ImdbRecordPager;

use crate::core::{record_viewer_operation::RecordViewerOperation, view_mode::RecordViewerMode};

pub struct RecordViewer<T>
where
    T: ImdbRecordPager,
{
    // config: RecordViewerConfig,
    pager: T,
}

impl<T> RecordViewer<T>
where
    T: ImdbRecordPager,
    T::ReadStorageEntryType: Display,
{
    pub fn new(pager: T) -> Self {
        RecordViewer {
            // config,
            pager: pager,
        }
    }

    pub fn execute(&mut self, operation: RecordViewerOperation) {
        match operation.viewer_mode {
            RecordViewerMode::AllRecords => self.view_all_records(),
            RecordViewerMode::NextRecord => todo!("NextRecord is not yet supported"),
            RecordViewerMode::RecordByKey => todo!("RecordByKey is not yet supported"),
        }
    }

    pub fn view_all_records(&mut self) {
        loop {
            let record = self.pager.load_next_record_and_metadata();
            if record.is_err() {
                eprintln!("Error while loading record: {:?}", record.err().unwrap());
                break;
            }
            let record = record.unwrap();
            if record.is_none() {
                break;
            }
            let record = record.unwrap();
            eprintln!("Record: {}", record);
        }
    }
}
