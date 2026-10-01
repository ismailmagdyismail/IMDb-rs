use crate::core::storage::{
    imdb_storage_entries::Offset, imdb_storage_operations_status::ImdbStorageError,
};

pub trait ImdbRecordPager {
    type ReadStorageEntryType;

    fn load_next_record_and_metadata(
        &mut self,
    ) -> Result<Option<Self::ReadStorageEntryType>, ImdbStorageError>;

    fn load_specific_record_and_meta_data_using_id_offset(
        &mut self,
        offset: Offset,
    ) -> Result<Option<Self::ReadStorageEntryType>, ImdbStorageError>;
}
