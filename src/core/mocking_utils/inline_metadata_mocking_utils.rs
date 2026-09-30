use crate::core::storage::{
    imdb_inline_metadata_storage_engine::imdb_inline_metadata_storage_record::{
        INLINE_STORAGE_RECORD_HEADER_SIZE, ImdbInlineMetaDataStorageRecord,
    },
    pager::Offset,
};

pub fn find_record_offset(
    records: &Vec<ImdbInlineMetaDataStorageRecord>,
    index: usize,
) -> (u64, u64) {
    let header_offset_expected = records.iter().enumerate().fold(0, |prev, (i, entry)| {
        if i >= index {
            return prev;
        }
        return prev + INLINE_STORAGE_RECORD_HEADER_SIZE + entry.record.ser_size();
    });
    let payload_offset_expected = header_offset_expected + INLINE_STORAGE_RECORD_HEADER_SIZE;
    return (
        header_offset_expected as u64,
        payload_offset_expected as u64,
    );
}

pub fn find_record_offset_with_starting_offset(
    records: &Vec<ImdbInlineMetaDataStorageRecord>,
    index: usize,
    starting_offset: Offset,
) -> (u64, u64) {
    let starting_offset = records
        .iter()
        .enumerate()
        .fold(starting_offset, |prev, (i, entry)| {
            if i >= index {
                return prev;
            }
            return prev
                + INLINE_STORAGE_RECORD_HEADER_SIZE as u64
                + entry.record.ser_size() as u64;
        });
    let record_offset_expected = starting_offset + INLINE_STORAGE_RECORD_HEADER_SIZE as u64;
    return (starting_offset as u64, record_offset_expected as u64);
}
