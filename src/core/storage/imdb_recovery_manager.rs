use crate::core::{
    index::imdb_index::ImdbIndexWriter,
    storage::{
        imdb_disk_records_manager::ImdbDiskRecordsManager,
        imdb_storage_entries::ImdbStorageWriteEntry,
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
                index.cache_record(
                    storage_entry.record,
                    ImdbStorageWriteEntry {
                        identfying_offset: storage_entry.identfying_offset,
                    },
                );
            } else {
                break;
            }
        }
        return Ok(());
    }
}
