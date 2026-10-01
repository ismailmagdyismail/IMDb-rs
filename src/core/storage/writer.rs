use crate::core::{
    record::imdb_record::ImdbRecord, storage::imdb_storage_operations_status::ImdbStorageError,
};

pub trait ImdbRecordWriter {
    type WriteStorageEntry;

    fn append_record(
        &mut self,
        record: &ImdbRecord,
    ) -> Result<Self::WriteStorageEntry, ImdbStorageError>;

    fn sync(&mut self) -> Result<(), ImdbStorageError>;
}
