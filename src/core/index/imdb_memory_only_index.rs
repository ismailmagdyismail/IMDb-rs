use crate::core::{
    record::imdb_record::{ImdbRecord, ImdbRecordKey},
    storage::pager::{ImdbRecordPager, Offset},
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ImdbMemoryOnlyIndex {
    kv_offset_index: HashMap<ImdbRecordKey, Offset>,
}

impl ImdbMemoryOnlyIndex {
    pub fn new() -> Result<ImdbMemoryOnlyIndex, String> {
        let index = ImdbMemoryOnlyIndex {
            kv_offset_index: HashMap::new(),
        };

        Ok(index)
    }

    pub fn load_all_index<T>(&mut self, pager: &mut T) -> Result<(), String>
    where
        T: ImdbRecordPager,
    {
        loop {
            if let Some(storage_entry) = pager.load_next_record_and_metadata()? {
                self.kv_offset_index
                    .insert(storage_entry.record.key, storage_entry.identfying_offset);
            } else {
                break;
            }
        }
        return Ok(());
    }

    pub fn read_record_offset(&mut self, key: &ImdbRecordKey) -> Option<&u64> {
        self.kv_offset_index.get(key)
    }

    // depricated
    // couples access to offset with fetching record
    // use "read_record_offset" api for more granularity
    // callers may have result cached in some BufferPool so this couples index with disk access
    // up tp caller to coordinate that
    pub fn read_record<T>(
        &self,
        key: &ImdbRecordKey,
        pager: &mut T,
    ) -> Result<Option<ImdbRecord>, String>
    where
        T: ImdbRecordPager,
    {
        let offset = match self.kv_offset_index.get(key) {
            Some(offset) => offset,
            None => return Result::Ok(Option::None),
        };
        let storage_record = pager.load_specific_record_and_meta_data_using_id_offset(*offset)?;
        if let Option::Some(storage_record) = storage_record {
            return Result::Ok(Option::Some(storage_record.record));
        }
        let fmt_error = format!("[Imdb Index Error]: record  found in index, not on Disk");
        return Result::Err(fmt_error);
    }

    pub fn write_record(&mut self, key: ImdbRecordKey, offset: Offset) {
        self.kv_offset_index.insert(key, offset);
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::core::{
        index::imdb_memory_only_index::ImdbMemoryOnlyIndex,
        mocking_utils::records_paging::{create_kv_entry, write_mock_records},
        storage::imdb_inline_metadata_pager::ImdbInlineMetaDataPager,
    };

    #[test]
    fn test_populating_whole_index() {
        let index_path: &Path = Path::new("memory_only_index_populating_test.bin");
        write_mock_records(&index_path, 100);
        let mut pager = ImdbInlineMetaDataPager::new(&index_path).unwrap();
        let mut index = ImdbMemoryOnlyIndex::new().unwrap();
        index.load_all_index(&mut pager).unwrap();
        assert_eq!(index.kv_offset_index.keys().count(), 100);
    }

    #[test]
    fn test_random_index_read() {
        let path = Path::new("memory_only_index_random_read_test.bin");
        let iteartion = 100;
        write_mock_records(path, iteartion);

        let mut pager = ImdbInlineMetaDataPager::new(path).unwrap();
        let mut index = ImdbMemoryOnlyIndex::new().unwrap();
        // populate all of the index
        index.load_all_index(&mut pager).unwrap();

        // test existing record
        let (key, value) = create_kv_entry(1);
        let key = key.into_bytes().to_owned();
        let record = index.read_record(&key, &mut pager).unwrap();
        assert!(record.is_some());
        let record = record.unwrap();
        assert_eq!(record.key, key);
        assert_eq!(record.value, value.into_bytes().to_owned());

        // test not existing record
        let (key, _) = create_kv_entry(iteartion + 10);
        let key = key.into_bytes().to_owned();
        let record = index.read_record(&key, &mut pager).unwrap();
        assert!(record.is_none());
    }
}
