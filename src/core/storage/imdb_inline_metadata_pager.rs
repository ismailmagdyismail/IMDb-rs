use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader},
    path::Path,
};

use crate::core::{
    record::imdb_record::{ImdbRecord, ImdbRecordMetaData},
    storage::imdb_inline_metadata_format::decode_record,
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
    buf_io: BufReader<File>,
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
            buf_io: BufReader::new(file),
        })
    }

    pub fn load_record(&mut self) -> Result<Option<(ImdbRecordMetaData, ImdbRecord)>, String> {
        // buffered IO handles sliding window and proxies any needed byte fetching Requests to the Disk-IO
        // N bytes are consumed (HEADER, Payload) then window moves over the decoded size
        // if buffer is empty, a request to fetch N KBytes to the disk is made, cached in memory
        // total of 2 copies are made
        //  1- Os Page cache into internal buffer of BufReader
        //  2- Copy from BufReader internal buffer, into the newly create struct / Record [I think this can be removed]
        // if EOF is reached, internal buffer is empty
        let buffer = self.buf_io.fill_buf().map_err(|err: std::io::Error| {
            let formatted_error = format!("[Imdb Error happend while loading record]: {}", err);
            formatted_error
        })?;
        if buffer.is_empty() {
            return Ok(None);
        }
        // deserialize
        let (metadata, record, decoded_size) = decode_record(buffer)?;
        // slide the window over
        self.buf_io.consume(decoded_size as usize);

        Ok(Some((metadata, record)))
    }
}

impl Iterator for ImdbInlineMetaDataPager {
    type Item = Result<(ImdbRecordMetaData, ImdbRecord), String>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.load_record() {
            Result::Ok(optional_record) => {
                if let Option::Some((metadata, record)) = optional_record {
                    return Option::Some(Result::Ok((metadata, record)));
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
            if let Some((_, record)) = res {
                assert!(verify_record(loaded_records_count, &record));
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
            let (_, record) = result.unwrap();
            assert!(verify_record(i, &record));
        }
    }
}
