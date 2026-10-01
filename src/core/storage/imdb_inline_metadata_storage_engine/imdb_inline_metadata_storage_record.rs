use crate::core::record::imdb_record::{
    ImdbRecord, ImdbRecordMetaData, KEY_LEN_SIZE, VAL_LEN_SIZE,
};

pub struct ImdbInlineMetaDataStorageRecord {
    pub check_sum: u32,
    pub metadata: ImdbRecordMetaData,
    pub record: ImdbRecord,
}
pub const CHECK_SUM_SIZE: u32 = 4;
pub const INLINE_STORAGE_RECORD_HEADER_SIZE: u32 = CHECK_SUM_SIZE + KEY_LEN_SIZE + VAL_LEN_SIZE;
