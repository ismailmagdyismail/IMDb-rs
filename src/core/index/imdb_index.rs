use crate::core::storage::pager::ImdbRecordMetadataStorageEntry;

// interface for writing to the index
// takes the storage Entry returned from the storage layer below
// implementation decides what to cache
pub trait ImdbIndexWriter {
    fn cache_record(&mut self, storage_entry: ImdbRecordMetadataStorageEntry);
}

// till we figure out a good abstraction
// pub trait ImdbIndexReader {
// }
