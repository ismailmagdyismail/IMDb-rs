use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Seek},
    path::Path,
};

use crate::core::storage::{
    imdb_inline_metadata_format::decode_record,
    pager::{ImdbRecordPager, Offset, RecordMetadataStorageEntry},
};

/*
- Meta-Data for each record is stored inline with the record itself
- only 1 file (data-file/records-file/payload-file) is accessed to fetch the files
+---------------+--------------+----------------+------------+---------------+
| checksum[u32] | key_len[u32] | value_len[u32] | key_payload| value_payload |
+---------------+--------------+----------------+------------+---------------+
*/

#[derive(Debug)]
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

    pub fn read_next_record_and_meta_data(
        &mut self,
    ) -> Result<Option<RecordMetadataStorageEntry>, String> {
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
            identfying_offset: metadata_offset,
        };
        Ok(Some(entry))
    }

    pub fn read_specific_record_and_meta_data(&self) {}
}

impl ImdbRecordPager for ImdbInlineMetaDataPager {
    fn load_next_record_and_metadata(
        &mut self,
    ) -> Result<Option<RecordMetadataStorageEntry>, String> {
        return self.read_next_record_and_meta_data();
    }

    fn load_specific_record_and_meta_data_using_id_offset(
        &self,
        offset: Offset,
    ) -> Result<Option<RecordMetadataStorageEntry>, String> {
        Err("".to_string())
    }
}

impl Iterator for ImdbInlineMetaDataPager {
    type Item = Result<RecordMetadataStorageEntry, String>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.read_next_record_and_meta_data() {
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
    use std::path::Path;

    use crate::core::{
        mocking_utils::records_paging::{verify_record, write_mock_records},
        storage::imdb_inline_metadata_pager::ImdbInlineMetaDataPager,
    };

    #[test]
    pub fn test_loading_records() {
        let file_path = Path::new("data_test.bin");
        let records_count = 100;
        write_mock_records(file_path, records_count);

        let mut pager = ImdbInlineMetaDataPager::new(file_path).unwrap();
        let mut loaded_records_count = 0;

        loop {
            let res = pager.read_next_record_and_meta_data().unwrap();
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
