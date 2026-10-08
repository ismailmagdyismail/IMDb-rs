use std::fmt::Display;

use crate::core::storage::imdb_inline_metadata_storage_engine::imdb_inline_metadata_storage_entries::ImdbInlineMetaDataStorageReadEntry;

impl Display for ImdbInlineMetaDataStorageReadEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ImdbInlineMetaDataStorageReadEntry {{ metadata: {:?}, record: {:?}, checksum: {}, checksum_offset: {}, metadata_offset: {}, record_offset: {}, identfying_offset: {} }}",
            self.metadata,
            self.record,
            self.checksum,
            self.checksum_offset,
            self.metadata_offset,
            self.record_offset,
            self.identfying_offset
        )
    }
}
