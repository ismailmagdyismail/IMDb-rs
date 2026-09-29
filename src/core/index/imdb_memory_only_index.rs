use crate::core::{
    record::imdb_record::{ImdbRecord, ImdbRecordKey},
    storage::pager::{ImdbRecordPager, Offset},
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ImdbMemoryOnlyIndex<T>
where
    T: ImdbRecordPager,
{
    kv_offset_index: HashMap<ImdbRecordKey, Offset>,
    index_pager: T,
}

impl<T> ImdbMemoryOnlyIndex<T>
where
    T: ImdbRecordPager,
{
    pub fn new(index_pager: T) -> Result<ImdbMemoryOnlyIndex<T>, String> {
        let index = ImdbMemoryOnlyIndex {
            kv_offset_index: HashMap::new(),
            index_pager,
        };

        Ok(index)
    }

    pub fn load_all_index(&mut self) -> Result<(), String> {
        loop {
            if let Some(storage_entry) = self.index_pager.load_next_record_and_metadata()? {
                self.kv_offset_index
                    .insert(storage_entry.record.key, storage_entry.identfying_offset);
            } else {
                break;
            }
        }
        return Ok(());
    }

    pub fn read_record(&mut self, key: &ImdbRecordKey) -> Result<Option<ImdbRecord>, String> {
        let offset = match self.kv_offset_index.get(key) {
            Some(offset) => offset,
            None => return Result::Ok(Option::None),
        };
        let storage_record = self
            .index_pager
            .load_specific_record_and_meta_data_using_id_offset(*offset)?;
        if let Option::Some(storage_record) = storage_record {
            return Result::Ok(Option::Some(storage_record.record));
        }
        return Result::Ok(Option::None);
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
        let pager = ImdbInlineMetaDataPager::new(&index_path).unwrap();
        let mut index = ImdbMemoryOnlyIndex::new(pager).unwrap();
        index.load_all_index().unwrap();
        assert_eq!(index.kv_offset_index.keys().count(), 100);
    }

    #[test]
    fn test_random_index_read() {
        let path = Path::new("memory_only_index_random_read_test.bin");
        let iteartion = 100;
        write_mock_records(path, iteartion);

        let pager = ImdbInlineMetaDataPager::new(path).unwrap();
        let mut index = ImdbMemoryOnlyIndex::new(pager).unwrap();
        // populate all of the index
        index.load_all_index().unwrap();

        // test existing record
        let (key, value) = create_kv_entry(1);
        let key = key.into_bytes().to_owned();
        let record = index.read_record(&key).unwrap();
        assert!(record.is_some());
        let record = record.unwrap();
        assert_eq!(record.key, key);
        assert_eq!(record.value, value.into_bytes().to_owned());

        // test not existing record
        let (key, _) = create_kv_entry(iteartion + 10);
        let key = key.into_bytes().to_owned();
        let record = index.read_record(&key).unwrap();
        assert!(record.is_none());
    }
}
