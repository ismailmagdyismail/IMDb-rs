use std::path::Path;

use crate::core::{
    index::imdb_primary_index::ImdbPrimaryIndex,
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
    index: ImdbPrimaryIndex,
    recovery_manager: RecoveryManager,
}

impl ImdbStorageEngine {
    pub fn new(storage_path: &Path) -> Result<ImdbStorageEngine, String> {
        ImdbStorageEngine::init_storage_directory(storage_path)?;
        let disk_manager = ImdbInlineMetaDataDiskManager::new(storage_path)?;
        let index = ImdbPrimaryIndex::new()?;
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

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::core::{
        mocking_utils::records_paging::{create_kv_entry, create_records},
        record::imdb_record::ImdbRecord,
        storage::imdb_storage_engine::ImdbStorageEngine,
    };

    fn create_test_dir(suffix: &str) -> String {
        let dir_path = Path::new("storage_engine_unit_tests");
        let mut dir_path = dir_path.to_path_buf();
        dir_path.push(suffix);
        std::fs::create_dir_all(&dir_path).unwrap();
        let path: String = dir_path.to_str().to_owned().unwrap().to_string();
        return path;
    }

    #[test]
    fn test_writing() {
        let path = create_test_dir("writing_test");
        let mut storage_engine: ImdbStorageEngine =
            ImdbStorageEngine::new(Path::new(&path)).unwrap();
        let records = create_records(100);
        for (record, _) in records {
            let res = storage_engine.write_record(record);
            assert!(res.is_ok());
            res.unwrap();
        }
    }

    fn verify_read_record(actual: &ImdbRecord, expected: &ImdbRecord) {
        assert_eq!(actual.key, expected.key);
        assert_eq!(actual.value, expected.value);
    }

    #[test]
    fn test_reading_existing_records() {
        let path = create_test_dir("reading_test");
        let mut storage_engine: ImdbStorageEngine =
            ImdbStorageEngine::new(Path::new(&path)).unwrap();
        let records = create_records(100);
        for (record, _) in records {
            let res = storage_engine.write_record(record);
            assert!(res.is_ok());
            res.unwrap();
        }
        let records = create_records(100);
        for (record, _) in records {
            let read_record = storage_engine.read_record(record.key.clone()).unwrap();
            assert!(read_record.is_some());
            verify_read_record(&record, &read_record.unwrap());
        }
    }

    #[test]
    fn test_loading_all_records() {
        let path = create_test_dir("loading_records_test");

        {
            let mut storage_engine: ImdbStorageEngine =
                ImdbStorageEngine::new(Path::new(&path)).unwrap();
            let records = create_records(100);
            for (record, _) in records {
                let res = storage_engine.write_record(record);
                assert!(res.is_ok());
                res.unwrap();
            }
        }

        let mut storage_engine: ImdbStorageEngine =
            ImdbStorageEngine::new(Path::new(&path)).unwrap();
        let records_clone = create_records(100);
        for (record, _) in records_clone {
            let read_record = storage_engine.read_record(record.key.clone()).unwrap();
            assert!(read_record.is_some());
            let read_record = read_record.unwrap();
            verify_read_record(&record, &read_record);
        }
    }

    #[test]
    fn test_write_destruct_load_read_update_read() {
        let path = create_test_dir("write_then_read_after_load");

        // write then destroy
        {
            let mut storage_engine: ImdbStorageEngine =
                ImdbStorageEngine::new(Path::new(&path)).unwrap();
            let records = create_records(100);
            for (record, _) in records {
                let res = storage_engine.write_record(record);
                assert!(res.is_ok());
                res.unwrap();
            }
        }

        // load
        let mut storage_engine: ImdbStorageEngine =
            ImdbStorageEngine::new(Path::new(&path)).unwrap();

        // read
        let records_clone = create_records(100);
        for (record, _) in records_clone {
            let read_record = storage_engine.read_record(record.key.clone()).unwrap();
            assert!(read_record.is_some());
            let read_record = read_record.unwrap();
            verify_read_record(&record, &read_record);
        }

        // update
        let records = create_records(100);
        for (i, (mut record, _)) in records.into_iter().enumerate() {
            let (_, val) = create_kv_entry(i);
            record.value = (val + "_updated ").as_bytes().to_vec();
            let res = storage_engine.write_record(record);
            assert!(res.is_ok());
            res.unwrap();
        }

        // read again
        let records_clone = create_records(100);
        for (i, (mut record, _)) in records_clone.into_iter().enumerate() {
            let (_, val) = create_kv_entry(i);
            record.value = (val + "_updated ").as_bytes().to_vec();
            let res = storage_engine.read_record(record.key.clone());
            assert!(res.is_ok());
            let read_record = res.unwrap();
            assert!(read_record.is_some());
            let read_record = read_record.unwrap();
            println!(
                "read key:: {}",
                String::from_utf8(read_record.key.clone()).unwrap()
            );
            println!(
                "actual key:: {}",
                String::from_utf8(record.key.clone()).unwrap()
            );
            println!(
                "read value:: {}",
                String::from_utf8(read_record.value.clone()).unwrap()
            );
            println!(
                "actual value:: {}",
                String::from_utf8(record.value.clone()).unwrap()
            );
            verify_read_record(&record, &read_record);
        }
    }
}
