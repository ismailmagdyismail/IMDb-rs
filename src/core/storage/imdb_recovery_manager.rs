use std::path::Path;

use crate::core::{
    index::{imdb_index::ImdbIndexWriter, imdb_memory_only_index::ImdbMemoryOnlyIndex},
    mocking_utils::records_paging::{create_kv_entry, write_mock_records},
    storage::{
        imdb_disk_records_manager::ImdbDiskRecordsManager,
        imdb_inline_metadata_storage_engine::{
            imdb_inline_metadata_disk_manager::ImdbInlineMetaDataDiskManager,
            imdb_inline_metadata_format::encode_record,
        },
    },
};

pub struct RecoveryManager {}

impl RecoveryManager {
    pub fn recover<DiskManager, IndexManager>(
        &self,
        disk_manager: &mut DiskManager,
        index: &mut IndexManager,
    ) -> Result<(), String>
    where
        DiskManager: ImdbDiskRecordsManager,
        IndexManager: ImdbIndexWriter,
    {
        loop {
            if let Some(storage_entry) = disk_manager.read_next_record()? {
                index.cache_record(storage_entry);
            } else {
                break;
            }
        }
        return Ok(());
    }
}
