use std::path::Path;

use crate::core::{
    index::imdb_memory_only_index::ImdbMemoryOnlyIndex,
    record::imdb_record::{ImdbRecord, ImdbRecordKey},
    storage::{
        imdb_disk_records_manager::ImdbDiskRecordsManager,
        imdb_inline_metadata_storage_engine::imdb_inline_metadata_disk_manager::ImdbInlineMetaDataDiskManager,
        imdb_recovery_manager::RecoveryManager,
    },
};

/*
    yet to figure out abstraction level:
    A. Make ImdbStorageEngine<T: ImdbDiskManager, U: ImdbIndex>
        + becomes a framework like
        + could easily add new implementations
        - more rigid towards un-usual scenarios
    B. Make ImdbInlineMetaDataStorageEngine and ImdbSeperateMetaDataStorageEngine
        + could change core execution of the engine
        + gives more flexibility in changing engine
        - a bit more duplication in basic cases
*/
pub struct ImdbStorageEngine {
    disk_manager: ImdbInlineMetaDataDiskManager,
    index: ImdbMemoryOnlyIndex,
    recovery_manager: RecoveryManager,
}

impl ImdbStorageEngine {
    pub fn new(storage_path: &Path) -> Result<ImdbStorageEngine, String> {
        ImdbStorageEngine::init_storage_directory(storage_path)?;
        let disk_manager = ImdbInlineMetaDataDiskManager::new(storage_path)?;
        let index = ImdbMemoryOnlyIndex::new()?;
        let recovery_manager = RecoveryManager {};
        let mut storage_engine: ImdbStorageEngine = ImdbStorageEngine {
            disk_manager,
            index,
            recovery_manager,
        };
        storage_engine.load_all()?;
        Ok(storage_engine)
    }

    pub fn load_all(&mut self) -> Result<(), String> {
        self.recovery_manager
            .recover(&mut self.disk_manager, &mut self.index)?;
        Ok(())
    }

    pub fn write_record(&mut self, record: ImdbRecord) -> Result<(), String> {
        let storage_entry = self.disk_manager.write_record(&record)?;
        self.disk_manager.sync()?;
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
        let storage_record = self.disk_manager.read_record_with_id_offset(*offset)?;
        if let Some(storage_record) = storage_record {
            Result::Ok(Some(storage_record.record))
        } else {
            // record found in index, but not on disk !!
            // this means in-consistency between index, on disk storage
            let fmt_error = format!("[Imdb Storage Error]: record found in index, not on Disk");
            Result::Err(fmt_error)
        }
    }

    // creates directory (with all of its missing parents)
    // if directory already exists, no changes occur
    fn init_storage_directory(dir_path: &Path) -> Result<(), String> {
        std::fs::create_dir_all(dir_path).map_err(|err| {
            let fmt_error = format!(
                "[Imdb Directory]: error happend while createing Imdb directory {} ",
                err
            );
            fmt_error
        })?;
        Ok(())
    }
}
