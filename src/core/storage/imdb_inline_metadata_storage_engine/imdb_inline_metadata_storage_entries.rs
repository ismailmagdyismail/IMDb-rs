use crate::core::{
    record::imdb_record::{ImdbRecord, ImdbRecordMetaData},
    storage::imdb_storage_entries::Offset,
};

pub struct ImdbInlineMetaDataStorageReadEntry {
    pub metadata: ImdbRecordMetaData,
    pub record: ImdbRecord,

    pub record_offset: Offset,
    pub metadata_offset: Offset,

    pub identfying_offset: Offset,
}

pub struct ImdbInlineMetaDataStorageWriteEntry {
    pub record_offset: Offset,
    pub metadata_offset: Offset,

    pub identfying_offset: Offset,
}
