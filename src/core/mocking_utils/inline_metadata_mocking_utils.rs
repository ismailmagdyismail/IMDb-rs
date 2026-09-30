use crate::core::{
    record::imdb_record::{ImdbRecord, ImdbRecordMetaData},
    storage::pager::Offset,
};

pub fn find_record_offset(
    records: &Vec<(ImdbRecord, ImdbRecordMetaData)>,
    index: usize,
) -> (u64, u64) {
    let metadata_offset_expected = records.iter().enumerate().fold(0, |prev, (i, entry)| {
        if i >= index {
            return prev;
        }
        let (record, metadata) = entry;
        return prev + metadata.ser_size() + record.ser_size();
    });
    let record_offset_expected = metadata_offset_expected + records[index].1.ser_size();
    return (
        metadata_offset_expected as u64,
        record_offset_expected as u64,
    );
}

pub fn find_record_offset_with_starting_offset(
    records: &Vec<(ImdbRecord, ImdbRecordMetaData)>,
    index: usize,
    starting_offset: Offset,
) -> (u64, u64) {
    let metadata_offset_expected =
        records
            .iter()
            .enumerate()
            .fold(starting_offset, |prev, (i, entry)| {
                if i >= index {
                    return prev;
                }
                let (record, metadata) = entry;
                return prev + metadata.ser_size() as u64 + record.ser_size() as u64;
            });
    let record_offset_expected = metadata_offset_expected + records[index].1.ser_size() as u64;
    return (
        metadata_offset_expected as u64,
        record_offset_expected as u64,
    );
}
