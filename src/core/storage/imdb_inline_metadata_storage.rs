use std::path::Path;

use crate::core::{
    index::imdb_memory_only_index::ImdbMemoryOnlyIndex, record::imdb_record::{ImdbRecord, ImdbRecordKey, ImdbRecordMetaData}, storage::{
        imdb_inline_metadata_pager::ImdbInlineMetaDataPager, imdb_inline_metadata_writer::ImdbInlineMetaDataWriter, pager::ImdbRecordPager, writer::ImdbRecordWriter,
    },
};

pub const IMDB_INLINE_METADATA_RECORDS_FILE_NAME: &'static str = "imdb_inline_metadata.bin";
pub struct ImdbInlineMetaDataStorage {
    pager: ImdbInlineMetaDataPager,
    writer: ImdbInlineMetaDataWriter,

    // I am not sure yet about the abtraction level here
    // should Index be:
    //      A. a part of the storage layer ?
    //
    //
    //
    //
    //
    //
    //
    //      B. a higher layer that makes use of storage layer
    //
    //
    //
    //
    //
    //
    //
    //      C. a Sibiling layer, which is coordinated with storage layer via another
    //      3rd layer that sits on top of them both
    //
    //
    //
    //
    //
    //
    //
    index: ImdbMemoryOnlyIndex,
}

impl ImdbInlineMetaDataStorage {
    pub fn new(dir_path: &Path) -> Result<ImdbInlineMetaDataStorage, String> {
        let mut file_path = dir_path.to_path_buf();
        file_path.push(IMDB_INLINE_METADATA_RECORDS_FILE_NAME);
        let file_path = Path::new(&file_path);

        let pager = ImdbInlineMetaDataPager::new(file_path)?;
        let writer = ImdbInlineMetaDataWriter::new(file_path)?;
        let index = ImdbMemoryOnlyIndex::new()?;
        let mut storage = ImdbInlineMetaDataStorage {
            pager,
            writer,
            index,
        };
        storage.load()?;
        Ok(storage)
    }

    fn load(&mut self) -> Result<(), String> {
        self.index.load_all_index(&mut self.pager)?;
        Ok(())
    }

    pub fn write_record(&mut self, record: ImdbRecord) -> Result<(), String> {
        let metadata = ImdbRecordMetaData::from(&record);
        let storage_entry = self.writer.write_record(&metadata, &record)?;
        self.writer.sync()?;
        self.index
            .write_record(record.key, storage_entry.identfying_offset);
        Ok(())
    }

    pub fn read_record(&mut self, key: ImdbRecordKey) -> Result<Option<ImdbRecord>, String> {
        let offset = self.index.read_record_offset(&key);
        let offset = if let Some(offset) = offset {
            offset
        } else {
            // record not found in index, then doesn't exist
            return Ok(None);
        };
        let storage_record = self
            .pager
            .load_specific_record_and_meta_data_using_id_offset(*offset)?;
        if let Some(storage_record) = storage_record {
            Result::Ok(Some(storage_record.record))
        } else {
            // record found in index, but not on disk !!
            // this means in-consistency between index, on disk storage
            let fmt_error = format!("[Imdb Storage Error]: record  found in index, not on Disk");
            Result::Err(fmt_error)
        }
    }
}
