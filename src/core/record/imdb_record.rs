pub type ImdbRecordKey = Vec<u8>;
pub type ImdbRecordValue = Vec<u8>;

#[derive(Debug)]
pub struct ImdbRecord {
    pub key: ImdbRecordKey,
    pub value: ImdbRecordValue,
}

#[derive(Debug)]
pub struct ImdbRecordMetaData {
    pub key_len: u32,
    pub val_len: u32,
}

impl ImdbRecordMetaData {
    pub fn from(record: &ImdbRecord) -> ImdbRecordMetaData {
        let metadata = ImdbRecordMetaData {
            key_len: record.key.len() as u32,
            val_len: record.value.len() as u32,
        };
        metadata
    }
}

pub const KEY_LEN_SIZE: u32 = 4;
pub const VAL_LEN_SIZE: u32 = 4;
pub const META_DATA_SIZE: u32 = KEY_LEN_SIZE + VAL_LEN_SIZE;
