use std::path::Path;

use crate::core::{
    record::imdb_record::ImdbRecord,
    storage::{
        imdb_disk_records_manager::ImdbDiskRecordsManager,
        imdb_inline_metadata_storage_engine::{
            imdb_inline_metadata_pager::ImdbInlineMetaDataPager,
            imdb_inline_metadata_writer::ImdbInlineMetaDataWriter,
        },
        imdb_storage_entries::{ImdbStorageReadEntry, ImdbStorageWriteEntry, Offset},
        pager::ImdbRecordPager,
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
        let inline_metadata_storage_entry = self.writer.write_record(record)?;
        let storage_entry = ImdbStorageWriteEntry {
            identfying_offset: inline_metadata_storage_entry.identfying_offset,
        };
        Ok(storage_entry)
    }

    fn sync(&mut self) -> Result<(), String> {
        self.writer.flush_and_fsync()?;
        Ok(())
    }

    fn read_record_with_id_offset(
        &mut self,
        offset: Offset,
    ) -> Result<Option<ImdbStorageReadEntry>, String> {
        let read_inline_metadata_storage_entry =
            self.pager.read_specific_record_and_meta_data(offset)?;
        if let Some(read_inline_metadata_storage_entry) = read_inline_metadata_storage_entry {
            let read_storage_entry = ImdbStorageReadEntry {
                identfying_offset: read_inline_metadata_storage_entry.identfying_offset,
                record: read_inline_metadata_storage_entry.record,
            };
            Ok(Some(read_storage_entry))
        } else {
            Ok(None)
        }
    }

    fn read_next_record(&mut self) -> Result<Option<ImdbStorageReadEntry>, String> {
        let next_inline_metadata_storage_read_entry = self.pager.load_next_record_and_metadata()?;
        if let Some(next_inline_metadata_storage_read_entry) =
            next_inline_metadata_storage_read_entry
        {
            let read_storage_entry = ImdbStorageReadEntry {
                identfying_offset: next_inline_metadata_storage_read_entry.identfying_offset,
                record: next_inline_metadata_storage_read_entry.record,
            };
            Ok(Some(read_storage_entry))
        } else {
            Ok(None)
        }
    }
}
