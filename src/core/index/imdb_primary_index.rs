use crate::core::{
    index::imdb_index::ImdbIndexWriter,
    record::imdb_record::{ImdbRecord, ImdbRecordKey},
    storage::imdb_storage_entries::{ImdbStorageWriteEntry, Offset},
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ImdbPrimaryIndex {
    kv_offset_index: HashMap<ImdbRecordKey, Offset>,
}

impl ImdbPrimaryIndex {
    pub fn new() -> Result<ImdbPrimaryIndex, String> {
        let index = ImdbPrimaryIndex {
            kv_offset_index: HashMap::new(),
        };

        Ok(index)
    }

    pub fn read_record_offset(&mut self, key: &ImdbRecordKey) -> Option<&u64> {
        self.kv_offset_index.get(key)
    }

    // depricated
    // couples access to offset with fetching record
    // use "read_record_offset" api for more granularity
    // callers may have result cached in some BufferPool so this couples index with disk access
    // up tp caller to coordinate that
    // pub fn read_record<T>(
    //     &self,
    //     key: &ImdbRecordKey,
    //     pager: &mut T,
    // ) -> Result<Option<ImdbRecord>, String>
    // where
    //     T: ImdbRecordPager

    pub fn write_record(&mut self, key: ImdbRecordKey, offset: Offset) {
        self.kv_offset_index.insert(key, offset);
    }
}

impl ImdbIndexWriter for ImdbPrimaryIndex {
    fn cache_record(&mut self, record: ImdbRecord, storage_write_entry: ImdbStorageWriteEntry) {
        self.write_record(record.key, storage_write_entry.identfying_offset);
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::core::{
        checksum::crc32::Crc32CheckSum,
        index::imdb_primary_index::ImdbPrimaryIndex,
        mocking_utils::records_paging::{create_kv_entry, write_mock_records},
        record::imdb_record::{ImdbRecord, ImdbRecordMetaData},
        storage::{
            imdb_inline_metadata_storage_engine::{
                imdb_inline_metadata_disk_manager::{
                    IMDB_INLINE_METADATA_RECORDS_FILE_NAME, ImdbInlineMetaDataDiskManager,
                },
                imdb_inline_metadata_format::encode_record,
            },
            imdb_recovery_manager::RecoveryManager,
        },
    };

    #[test]
    fn test_populating_whole_index() {
        let db_dir = Path::new("memory_only_index_populating_test");
        std::fs::create_dir_all(db_dir).unwrap();
        let mut db_data_file = db_dir.to_path_buf();
        db_data_file.push(IMDB_INLINE_METADATA_RECORDS_FILE_NAME);
        let db_data_file = Path::new(&db_data_file);

        let iterations = 100;
        write_mock_records(
            &db_data_file,
            iterations,
            &mut |record: &ImdbRecord, metadata: &ImdbRecordMetaData, buffer: &mut [u8]| {
                let checksum_calculator = Crc32CheckSum::new();
                encode_record(record, metadata, buffer, &checksum_calculator)?;
                Ok(())
            },
        );

        let mut disk_manager = ImdbInlineMetaDataDiskManager::new(db_dir).unwrap();
        let mut index = ImdbPrimaryIndex::new().unwrap();
        let recovery_manager = RecoveryManager {};

        recovery_manager
            .recover(&mut disk_manager, &mut index)
            .unwrap();
        assert_eq!(index.kv_offset_index.len(), iterations);
        for i in 0..iterations {
            let (key, _) = create_kv_entry(i);
            let offset = index.read_record_offset(&key.into_bytes());
            assert!(offset.is_some());
        }
    }
}
