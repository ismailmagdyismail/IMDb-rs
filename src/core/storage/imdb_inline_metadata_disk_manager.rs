use std::path::Path;

use crate::core::{
    record::imdb_record::{ImdbRecord, ImdbRecordMetaData},
    storage::{
        imdb_disk_records_manager::ImdbDiskRecordsManager,
        imdb_inline_metadata_pager::ImdbInlineMetaDataPager,
        imdb_inline_metadata_writer::ImdbInlineMetaDataWriter,
        pager::{ImdbRecordMetadataStorageEntry, ImdbRecordPager, Offset},
        writer::{ImdbRecordWriter, ImdbStorageEntry},
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
    type Reader = ImdbInlineMetaDataPager;
    type Writer = ImdbInlineMetaDataWriter;

    fn write_record(&mut self, record: &ImdbRecord) -> Result<ImdbStorageEntry, String> {
        let metadata = ImdbRecordMetaData::from(record);
        let storage_entry = self.writer.write_record(&metadata, record)?;
        Ok(storage_entry)
    }

    fn sync(&mut self) -> Result<(), String> {
        self.writer.sync()?;
        Ok(())
    }

    fn read_record_with_id_offset(
        &mut self,
        offset: Offset,
    ) -> Result<Option<ImdbRecordMetadataStorageEntry>, String> {
        return self
            .pager
            .load_specific_record_and_meta_data_using_id_offset(offset);
    }

    fn reader(&mut self) -> &mut Self::Reader {
        &mut self.pager
    }

    fn writer(&mut self) -> &mut Self::Writer {
        &mut self.writer
    }
}
