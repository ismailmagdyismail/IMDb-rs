use crate::core::{
    record::imdb_record::ImdbRecordKey,
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
                    .insert(storage_entry.record.key, storage_entry.record_offset);
            } else {
                break;
            }
        }
        return Ok(());
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::core::{
        index::imdb_memory_only_index::ImdbMemoryOnlyIndex,
        mocking_utils::records_paging::write_mock_records,
        storage::imdb_inline_metadata_pager::ImdbInlineMetaDataPager,
    };

    #[test]
    fn test_populating_whole_index() {
        let index_path: &Path = Path::new("data_index_test.bin");
        write_mock_records(&index_path, 100);
        let pager = ImdbInlineMetaDataPager::new(&index_path).unwrap();
        let mut index = ImdbMemoryOnlyIndex::new(pager).unwrap();
        index.load_all_index().unwrap();
        assert_eq!(index.kv_offset_index.keys().count(), 100);
    }
}
