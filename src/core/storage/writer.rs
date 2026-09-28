use crate::core::{
    record::imdb_record::{ImdbRecord, ImdbRecordMetaData},
    storage::pager::Offset,
};

#[derive(Debug)]
pub struct ImdbStorageEntry {
    // a bit of a leaky abstraction (exposing physical layout format) to return both to the caller
    pub record_offset: Offset,
    pub metadata_offset: Offset,

    // refer to pager.rs to
    pub identfying_offset: Offset,
}

pub trait ImdbRecordWriter {
    fn write_record_and_metadata(
        &mut self,
        metadata: &ImdbRecordMetaData,
        record: &ImdbRecord,
    ) -> Result<ImdbStorageEntry, String>;
}
