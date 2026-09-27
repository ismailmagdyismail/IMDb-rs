use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::Path,
};

use crate::core::storage::{
    buffer_helpers::advance_internal_buffer_cursor,
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
    buf_reader: BufReader<File>,               // for stateful iterative reads
    random_access_buf_reader: BufReader<File>, // for random / non sequential reads
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

        let stateless_read_file = file.try_clone().map_err(|error| {
            let formatted_errror = format!("[Imdb Opening Data file Error]: {}", error);
            return formatted_errror;
        })?;
        let random_access_buf_reader = BufReader::with_capacity(1024, stateless_read_file);

        Ok(ImdbInlineMetaDataPager {
            buf_reader: BufReader::new(file),
            random_access_buf_reader: random_access_buf_reader,
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

        let (metadata_offset, record_offset) =
            advance_internal_buffer_cursor(&mut self.buf_reader, metadata_size, record_size)?;

        let entry = RecordMetadataStorageEntry {
            metadata,
            record,
            record_offset,
            metadata_offset,
            identfying_offset: metadata_offset,
        };
        Ok(Some(entry))
    }

    pub fn read_specific_record_and_meta_data(
        &mut self,
        offset: Offset,
    ) -> Result<Option<RecordMetadataStorageEntry>, String> {
        self.random_access_buf_reader
            .seek(SeekFrom::Start(offset))
            .map_err(|err| {
                return format!("[Imdb Random Seek failure]: {}", err);
            })?;
        let buffer = self.random_access_buf_reader.fill_buf().map_err(|err| {
            return format!(
                "[Imdb Error happend while loading record with offset {}]: {}",
                offset, err
            );
        })?;
        if buffer.is_empty() {
            return Ok(None);
        }
        let (metadata, record, metadata_size, record_size) = decode_record(buffer)?;
        let (metadata_offset, record_offset) = advance_internal_buffer_cursor(
            &mut self.random_access_buf_reader,
            metadata_size,
            record_size,
        )?;
        let record_entry = RecordMetadataStorageEntry {
            record,
            metadata,
            record_offset,
            metadata_offset,
            identfying_offset: metadata_offset,
        };
        Ok(Some(record_entry))
    }
}

impl ImdbRecordPager for ImdbInlineMetaDataPager {
    fn load_next_record_and_metadata(
        &mut self,
    ) -> Result<Option<RecordMetadataStorageEntry>, String> {
        return self.read_next_record_and_meta_data();
    }

    fn load_specific_record_and_meta_data_using_id_offset(
        &mut self,
        offset: Offset,
    ) -> Result<Option<RecordMetadataStorageEntry>, String> {
        return self.read_specific_record_and_meta_data(offset);
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
        mocking_utils::records_paging::{verify_record, write_mock_records, write_records},
        record::imdb_record::{ImdbRecord, ImdbRecordMetaData},
        storage::{
            imdb_inline_metadata_pager::ImdbInlineMetaDataPager,
            pager::{ImdbRecordPager, RecordMetadataStorageEntry},
        },
    };

    #[test]
    pub fn test_loading_records() {
        let file_path = Path::new("inline_metadata_pager_loading_test.bin");
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
        let file_path = Path::new("inline_metadata_pager_iterator.bin");
        let records_count = 100;
        write_mock_records(file_path, records_count);

        let pager = ImdbInlineMetaDataPager::new(file_path).unwrap();

        for (i, result) in pager.enumerate() {
            let entry = result.unwrap();
            assert!(verify_record(i, &entry.record));
        }
    }

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

    pub fn verify_inline_metadata_fetched_storage_record_offsets(
        fetched_storage_record: Option<RecordMetadataStorageEntry>,
        original_records: &Vec<(ImdbRecord, ImdbRecordMetaData)>,
        record_to_verify_against_index: usize,
    ) {
        assert!(fetched_storage_record.is_some());
        let fetched_storage_record = fetched_storage_record.unwrap();
        let (metadata_offset_expected, record_offset_expected) =
            find_record_offset(original_records, record_to_verify_against_index);

        assert_eq!(
            fetched_storage_record.identfying_offset,
            fetched_storage_record.metadata_offset
        );
        assert_eq!(
            fetched_storage_record.metadata_offset,
            metadata_offset_expected as u64
        );
        assert_eq!(
            fetched_storage_record.record_offset,
            record_offset_expected as u64
        );
        assert_eq!(
            fetched_storage_record.metadata.key_len,
            original_records[record_to_verify_against_index].1.key_len
        );
        assert_eq!(
            fetched_storage_record.metadata.val_len,
            original_records[record_to_verify_against_index].1.val_len
        );
        assert_eq!(
            fetched_storage_record.record.key,
            original_records[record_to_verify_against_index].0.key
        );
        assert_eq!(
            fetched_storage_record.record.value,
            original_records[record_to_verify_against_index].0.value
        );
    }

    #[test]
    pub fn test_random_read() {
        let mut records = Vec::new();
        records.push((
            ImdbRecord {
                key: "1".as_bytes().to_vec(),
                value: "name".as_bytes().to_vec(),
            },
            ImdbRecordMetaData {
                key_len: 1,
                val_len: 4,
                check_sum: 0,
            },
        ));
        records.push((
            ImdbRecord {
                key: "123".as_bytes().to_vec(),
                value: "another_name".as_bytes().to_vec(),
            },
            ImdbRecordMetaData {
                key_len: 3,
                val_len: 12,
                check_sum: 0,
            },
        ));
        records.push((
            ImdbRecord {
                key: "123791237981273".as_bytes().to_vec(),
                value: "mashekjwhrkjwehrkjw".as_bytes().to_vec(),
            },
            ImdbRecordMetaData {
                key_len: 15,
                val_len: 19,
                check_sum: 0,
            },
        ));
        let path = Path::new("inline_metadata_pager_random_read.bin");
        write_records(&records, path);

        let mut pager = ImdbInlineMetaDataPager::new(&path).unwrap();

        for (i, _) in records.iter().enumerate() {
            let (metadata_offset, _) = find_record_offset(&records, i);
            let storage_record = pager
                .load_specific_record_and_meta_data_using_id_offset(metadata_offset)
                .unwrap();
            verify_inline_metadata_fetched_storage_record_offsets(storage_record, &records, i);
        }
    }
}
