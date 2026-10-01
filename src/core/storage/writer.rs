use crate::core::{
    record::imdb_record::ImdbRecord, storage::imdb_storage_entries::ImdbStorageWriteEntry,
};

pub trait ImdbRecordWriter {
    fn append_record(&mut self, record: &ImdbRecord) -> Result<ImdbStorageWriteEntry, String>;

    fn sync(&mut self) -> Result<(), String>;
}
