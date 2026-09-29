use crate::core::{
    record::imdb_record::{HEADER_SIZE, ImdbRecord, ImdbRecordMetaData},
    storage::imdb_inline_metadata_format::encode_record,
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

pub fn create_records(records_count: usize) -> Vec<(ImdbRecord, ImdbRecordMetaData)> {
    let mut vec = Vec::new();
    for i in 0..records_count {
        let (key, val) = create_kv_entry(i);

        let meta_data = ImdbRecordMetaData {
            check_sum: 0,
            key_len: key.as_bytes().len() as u32,
            val_len: val.as_bytes().len() as u32,
        };
        let record = ImdbRecord {
            key: key.into_bytes(),
            value: val.into_bytes(),
        };
        vec.push((record, meta_data));
    }

    return vec;
}

pub fn write_records(records: &Vec<(ImdbRecord, ImdbRecordMetaData)>, file_path: &Path) {
    let mut options = OpenOptions::new();
    let file = options
        .create(true)
        .truncate(true)
        .write(true)
        .open(file_path)
        .unwrap();
    let mut buf_writer = BufWriter::new(file);
    for (record, metadata) in records {
        let mut buffer = Vec::new();
        buffer.resize(
            HEADER_SIZE as usize + metadata.key_len as usize + metadata.val_len as usize,
            b'0',
        );
        encode_record(&record, &metadata, buffer.as_mut_slice()).unwrap();
        buf_writer.write_all(&buffer).unwrap();
    }
    buf_writer.flush().unwrap();
}

pub fn write_mock_records(file_path: &Path, records_count: usize) {
    let mut options = OpenOptions::new();
    let file = options
        .create(true)
        .truncate(true)
        .write(true)
        .open(file_path)
        .unwrap();
    let mut buf_writer = BufWriter::new(file);
    let records = create_records(records_count);
    for (record, meta_data) in records {
        let mut buffer = Vec::new();
        buffer.resize(
            HEADER_SIZE as usize + meta_data.key_len as usize + meta_data.val_len as usize,
            b'0',
        );
        encode_record(&record, &meta_data, buffer.as_mut_slice()).unwrap();
        buf_writer.write_all(&buffer).unwrap();
    }
    buf_writer.flush().unwrap();
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
