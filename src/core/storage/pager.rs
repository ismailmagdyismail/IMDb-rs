use crate::core::record::imdb_record::{ImdbRecord, ImdbRecordMetaData};

pub type Offset = u64;

pub struct RecordMetadataStorageEntry {
    pub metadata: ImdbRecordMetaData,
    pub record: ImdbRecord,
    pub record_offset: Offset,
    pub metadata_offset: Offset,
}

pub trait ImdbRecordPager {
    fn load_next_record_and_metadata(
        &mut self,
    ) -> Result<Option<RecordMetadataStorageEntry>, String>;
    // fn load_specific_record_and_metadata(
    //     &self,
    // ) -> Result<Option<(ImdbRecordMetaData, ImdbRecord)>, String>;
}

// pub trait ImdbRecordPager {
//     fn load_next_record(&mut self) -> Result<Option<ImdbRecord>, String>;
// }

// pub trait ImdbIndexPager {
//     fn load_next_metadata(&mut self) -> Result<Option<ImdbRecordMetaData>, String>;
// }
