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
            RecordViewerMode::NextRecord => self.view_next_record(),
            RecordViewerMode::ResetCursor => self.reset_viewer_cursor(),
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
                println!("No more records to display.");
                break;
            }
            let record = record.unwrap();
            println!("Record: {}", record);
        }
    }

    pub fn view_next_record(&mut self) {
        let record = self.pager.load_next_record_and_metadata();
        if record.is_err() {
            eprintln!("Error while loading record: {:?}", record.err().unwrap());
        } else {
            let record = record.unwrap();
            if record.is_some() {
                println!("Record: {}", record.unwrap());
            } else {
                println!("No more records to display.");
            }
        }
    }



    pub fn reset_viewer_cursor(&mut self) {
        println!("Resetting Cursor to the start.");
        self.pager.reset_cursor().unwrap_or_else(|err| {
            eprintln!("Error while resetting cursor: {:?}", err);
        });
    }
}
