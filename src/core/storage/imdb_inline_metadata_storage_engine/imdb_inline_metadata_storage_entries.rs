use crate::core::{
    record::imdb_record::{ImdbRecord, ImdbRecordMetaData},
    storage::imdb_storage_entries::Offset,
};

pub struct ImdbInlineMetaDataStorageReadEntry {
    pub metadata: ImdbRecordMetaData,
    pub record: ImdbRecord,
    pub checksum: u32,

    pub checksum_offset: Offset,
    pub metadata_offset: Offset,
    pub record_offset: Offset,

    pub identfying_offset: Offset,
}

pub struct ImdbInlineMetaDataStorageWriteEntry {
    pub checksum_offset: Offset,
    pub record_offset: Offset,
    pub metadata_offset: Offset,

    pub identfying_offset: Offset,
}
