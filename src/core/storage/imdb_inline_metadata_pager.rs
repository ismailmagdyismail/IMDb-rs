use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Seek},
    path::Path,
};

use crate::core::storage::{
    imdb_inline_metadata_format::decode_record,
    pager::{ImdbRecordPager, RecordMetadataStorageEntry},
};

/*
- Meta-Data for each record is stored inline with the record itself
- only 1 file (data-file/records-file/payload-file) is accessed to fetch the files
+---------------+--------------+----------------+------------+---------------+
| checksum[u32] | key_len[u32] | value_len[u32] | key_payload| value_payload |
+---------------+--------------+----------------+------------+---------------+
*/

pub struct ImdbInlineMetaDataPager {
    // data_file: File,
    buf_reader: BufReader<File>,
}

impl ImdbInlineMetaDataPager {
    pub fn new(file_path: &Path) -> Result<ImdbInlineMetaDataPager, String> {
        let mut options = OpenOptions::new();
        let file = options
            .read(true)
            .write(true)
            .create(true)
            .open(file_path)
            .map_err(|error| {
                let formatted_errror = format!("[Imdb Opening Data file Error]: {}", error);
                return formatted_errror;
            })?;
        Ok(ImdbInlineMetaDataPager {
            buf_reader: BufReader::new(file),
        })
    }

    pub fn load_record(&mut self) -> Result<Option<RecordMetadataStorageEntry>, String> {
        // buffered IO handles sliding window and proxies any needed byte fetching Requests to the Disk-IO
        // N bytes are consumed (HEADER, Payload) then window moves over the decoded size
        // if buffer is empty, a request to fetch N KBytes to the disk is made, cached in memory
        // total of 2 copies are made
        //  1- Os Page cache into internal buffer of BufReader
        //  2- Copy from BufReader internal buffer, into the newly create struct / Record [I think this can be removed]
        // if EOF is reached, internal buffer is empty
        let buffer = self.buf_reader.fill_buf().map_err(|err: std::io::Error| {
            let formatted_error = format!("[Imdb Error happend while loading record]: {}", err);
            formatted_error
        })?;
        if buffer.is_empty() {
            return Ok(None);
        }
        // deserialize
        let (metadata, record, metadata_size, record_size) = decode_record(buffer)?;

        // record offset of metadata
        // slide the window over by header
        let metadata_offset = self.buf_reader.stream_position().map_err(|err| {
            return format!("[Imdb Pager error while getting metadata offset]: {}", err);
        })?;
        self.buf_reader.consume(metadata_size as usize);

        // record offset of record
        // slide the window again to pass record
        let record_offset = self.buf_reader.stream_position().map_err(|err| {
            return format!("[Imdb Pager error while getting record offset]: {}", err);
        })?;
        self.buf_reader.consume(record_size as usize);

        let entry = RecordMetadataStorageEntry {
            metadata,
            record,
            record_offset,
            metadata_offset,
        };
        Ok(Some(entry))
    }
}

impl ImdbRecordPager for ImdbInlineMetaDataPager {
    fn load_next_record_and_metadata(
        &mut self,
    ) -> Result<Option<RecordMetadataStorageEntry>, String> {
        return self.load_record();
    }
}

impl Iterator for ImdbInlineMetaDataPager {
    type Item = Result<RecordMetadataStorageEntry, String>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.load_record() {
            Result::Ok(optional_record) => {
                if let Option::Some(entry) = optional_record {
                    return Option::Some(Result::Ok(entry));
                } else {
                    return Option::None;
                }
            }
            Result::Err(err) => Option::Some(Result::Err(err)),
        }
    }
}

#[cfg(test)]
mod test {
    use std::{
        fs::OpenOptions,
        io::{BufWriter, Write},
        path::Path,
    };

    use crate::core::{
        record::imdb_record::{HEADER_SIZE, ImdbRecord, ImdbRecordMetaData},
        storage::{
            imdb_inline_metadata_format::encode_record,
            imdb_inline_metadata_pager::ImdbInlineMetaDataPager,
        },
    };

    fn create_kv_entry(iteration: usize) -> (String, String) {
        let expected_key = iteration.to_string();
        let expected_val = String::from("val") + iteration.to_string().as_str();

        (expected_key, expected_val)
    }

    fn write_mock_records(file_path: &Path, records_count: usize) {
        let mut options = OpenOptions::new();
        let file = options
            .create(true)
            .truncate(true)
            .write(true)
            .open(file_path)
            .unwrap();
        let mut buf_writer = BufWriter::new(file);
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

    fn verify_record(iteration: usize, record: &ImdbRecord) -> bool {
        let (expected_key, expected_val) = create_kv_entry(iteration);
        return expected_key.into_bytes() == record.key
            && expected_val.into_bytes() == record.value;
    }

    #[test]
    pub fn test_loading_records() {
        let file_path = Path::new("data_test.bin");
        let records_count = 100;
        write_mock_records(file_path, records_count);

        let mut pager = ImdbInlineMetaDataPager::new(file_path).unwrap();
        let mut loaded_records_count = 0;

        loop {
            let res = pager.load_record().unwrap();
            if let Some(entry) = res {
                assert!(verify_record(loaded_records_count, &entry.record));
            } else {
                break;
            }
            loaded_records_count += 1;
        }
        assert_eq!(loaded_records_count, records_count);
    }

    #[test]
    pub fn test_loading_iterator() {
        let file_path = Path::new("data_test.bin");
        let records_count = 100;
        write_mock_records(file_path, records_count);

        let pager = ImdbInlineMetaDataPager::new(file_path).unwrap();

        for (i, result) in pager.enumerate() {
            let entry = result.unwrap();
            assert!(verify_record(i, &entry.record));
        }
    }
}
