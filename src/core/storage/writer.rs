use crate::core::record::imdb_record::ImdbRecord;

pub trait ImdbRecordWriter {
    type WriteStorageEntry;

    fn append_record(&mut self, record: &ImdbRecord) -> Result<Self::WriteStorageEntry, String>;

    fn sync(&mut self) -> Result<(), String>;
}
