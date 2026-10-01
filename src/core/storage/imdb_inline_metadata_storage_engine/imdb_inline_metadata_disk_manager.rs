use std::path::Path;

use crate::core::{
    record::imdb_record::ImdbRecord, storage::{
        imdb_disk_records_manager::ImdbDiskRecordsManager, imdb_inline_metadata_storage_engine::{
            imdb_inline_metadata_pager::ImdbInlineMetaDataPager,
            imdb_inline_metadata_writer::ImdbInlineMetaDataWriter,
        }, imdb_storage_entries::{ImdbStorageReadEntry, ImdbStorageWriteEntry, Offset}, pager::ImdbRecordPager, writer::ImdbRecordWriter,
    },
};

pub const IMDB_INLINE_METADATA_RECORDS_FILE_NAME: &'static str = "imdb_inline_metadata.bin";
pub struct ImdbInlineMetaDataDiskManager {
    pager: ImdbInlineMetaDataPager,
    writer: ImdbInlineMetaDataWriter,
}

impl ImdbInlineMetaDataDiskManager {
    pub fn new(dir_path: &Path) -> Result<ImdbInlineMetaDataDiskManager, String> {
        let mut file_path = dir_path.to_path_buf();
        file_path.push(IMDB_INLINE_METADATA_RECORDS_FILE_NAME);
        let file_path = Path::new(&file_path);

        let pager = ImdbInlineMetaDataPager::new(file_path)?;
        let writer = ImdbInlineMetaDataWriter::new(file_path)?;
        let storage = ImdbInlineMetaDataDiskManager { pager, writer };
        Ok(storage)
    }
}

impl ImdbDiskRecordsManager for ImdbInlineMetaDataDiskManager {
    fn write_record(&mut self, record: &ImdbRecord) -> Result<ImdbStorageWriteEntry, String> {
        let storage_entry = self.writer.append_record(record)?;
        Ok(storage_entry)
    }

    fn sync(&mut self) -> Result<(), String> {
        self.writer.sync()?;
        Ok(())
    }

    fn read_record_with_id_offset(
        &mut self,
        offset: Offset,
    ) -> Result<Option<ImdbStorageReadEntry>, String> {
        return self
            .pager
            .load_specific_record_and_meta_data_using_id_offset(offset);
    }

    fn read_next_record(&mut self) -> Result<Option<ImdbStorageReadEntry>, String> {
        self.pager.load_next_record_and_metadata()
    }
}
