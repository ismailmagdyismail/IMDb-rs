use crate::core::{
    record::imdb_record::ImdbRecord, storage::imdb_storage_entries::ImdbStorageWriteEntry,
};

// interface for writing to the index
// takes the storage Entry returned from the storage layer below
// implementation decides what to cache
pub trait ImdbIndexWriter {
    fn cache_record(&mut self, record: ImdbRecord, storage_write_entry: ImdbStorageWriteEntry);
}

// till we figure out a good abstraction
// pub trait ImdbIndexReader {
// }
