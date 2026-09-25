pub type ImdbRecordKey = Vec<u8>;
pub type ImdbRecordValue = Vec<u8>;

pub struct ImdbRecord {
    pub key: ImdbRecordKey,
    pub value: ImdbRecordValue,
}

pub struct ImdbRecordMetaData {
    pub check_sum: u32,
    pub key_len: u32,
    pub val_len: u32,
}
