use crate::core::{
    record::imdb_record::{ImdbRecord, ImdbRecordMetaData},
    storage::{
        imdb_inline_metadata_storage_engine::imdb_inline_metadata_storage_record::{
            INLINE_STORAGE_RECORD_HEADER_SIZE, ImdbInlineMetaDataStorageRecord,
        },
        imdb_storage_operations_status::ImdbStorageError,
    },
};
use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::Path,
};

pub fn create_kv_entry(iteration: usize) -> (String, String) {
    let expected_key = iteration.to_string();
    let expected_val = String::from("val") + iteration.to_string().as_str();

    (expected_key, expected_val)
}

pub fn create_records(records_count: usize) -> Vec<ImdbInlineMetaDataStorageRecord> {
    let mut vec = Vec::new();
    for i in 0..records_count {
        let (key, val) = create_kv_entry(i);

        let metadata = ImdbRecordMetaData {
            key_len: key.as_bytes().len() as u32,
            val_len: val.as_bytes().len() as u32,
        };
        let record = ImdbRecord {
            key: key.into_bytes(),
            value: val.into_bytes(),
        };
        vec.push(ImdbInlineMetaDataStorageRecord {
            record,
            metadata,
            check_sum: 0,
        });
    }

    return vec;
}

pub fn write_records<T>(
    records: &Vec<ImdbInlineMetaDataStorageRecord>,
    file_path: &Path,
    formatter_callback: &mut T,
) where
    T: FnMut(&ImdbRecord, &ImdbRecordMetaData, &mut [u8]) -> Result<(), ImdbStorageError>,
{
    let mut options = OpenOptions::new();
    let file = options
        .create(true)
        .truncate(true)
        .write(true)
        .open(file_path)
        .unwrap();
    let mut buf_writer = BufWriter::new(file);
    for storage_record in records {
        let mut buffer = Vec::new();
        buffer.resize(
            INLINE_STORAGE_RECORD_HEADER_SIZE as usize
                + storage_record.metadata.key_len as usize
                + storage_record.metadata.val_len as usize,
            b'0',
        );
        formatter_callback(
            &storage_record.record,
            &storage_record.metadata,
            buffer.as_mut_slice(),
        )
        .unwrap();
        buf_writer.write_all(&buffer).unwrap();
    }
    buf_writer.flush().unwrap();
    buf_writer.into_inner().unwrap().sync_all().unwrap();
}

pub fn write_mock_records<T>(file_path: &Path, records_count: usize, formatter_callback: &mut T)
where
    T: FnMut(&ImdbRecord, &ImdbRecordMetaData, &mut [u8]) -> Result<(), ImdbStorageError>,
{
    let mut options = OpenOptions::new();
    let file = options
        .create(true)
        .truncate(true)
        .write(true)
        .open(file_path)
        .unwrap();
    let mut buf_writer = BufWriter::new(file);
    let storage_records = create_records(records_count);
    for storage_record in storage_records {
        let mut buffer = Vec::new();
        buffer.resize(
            INLINE_STORAGE_RECORD_HEADER_SIZE as usize
                + storage_record.metadata.key_len as usize
                + storage_record.metadata.val_len as usize,
            b'0',
        );
        formatter_callback(
            &storage_record.record,
            &storage_record.metadata,
            buffer.as_mut_slice(),
        )
        .unwrap();
        buf_writer.write_all(&buffer).unwrap();
    }
    buf_writer.flush().unwrap();
    buf_writer.into_inner().unwrap().sync_all().unwrap();
}

pub fn verify_record(iteration: usize, record: &ImdbRecord) -> bool {
    let (expected_key, expected_val) = create_kv_entry(iteration);
    return expected_key.into_bytes() == record.key && expected_val.into_bytes() == record.value;
}

pub fn create_writer_file(file_path: &str) -> File {
    let mut options = OpenOptions::new();
    options
        .create(true)
        .write(true)
        .read(true)
        .truncate(true)
        .open(Path::new(file_path))
        .unwrap()
}
