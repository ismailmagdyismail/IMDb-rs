use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
    path::Path,
};

use crate::core::{
    record::imdb_record::META_DATA_SIZE,
    serdes::slicer::Slicer,
    storage::{
        buffer_helpers::{
            advance_internal_buffer_cursor_by_check_sum,
            advance_internal_buffer_cursor_by_metadata,
            advance_internal_buffer_cursor_by_record_payload,
        },
        imdb_inline_metadata_storage_engine::{
            imdb_inline_metadata_format::{
                decode_checksum, decode_metadata, decode_record_payload,
            },
            imdb_inline_metadata_storage_entries::ImdbInlineMetaDataStorageReadEntry,
            imdb_inline_metadata_storage_record::{
                CHECK_SUM_SIZE, INLINE_STORAGE_RECORD_HEADER_SIZE,
            },
        },
        imdb_storage_entries::Offset,
        pager::ImdbRecordPager,
    },
};

/*
- Meta-Data for each record is stored inline with the record itself
- only 1 file (data-file/records-file/payload-file) is accessed to fetch the files
+---------------+--------------+----------------+------------+---------------+
| checksum[u32] | key_len[u32] | value_len[u32] | key_payload| value_payload |
+---------------+--------------+----------------+------------+---------------+
*/

const RANDOM_BUFFER_READ_SIZE: usize = 1024;
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
        let random_access_buf_reader =
            BufReader::with_capacity(RANDOM_BUFFER_READ_SIZE, stateless_read_file);

        Ok(ImdbInlineMetaDataPager {
            buf_reader: BufReader::new(file),
            random_access_buf_reader: random_access_buf_reader,
        })
    }

    fn load_record(
        buf_reader: &mut BufReader<File>,
    ) -> Result<Option<ImdbInlineMetaDataStorageReadEntry>, String> {
        // Fill the internal buffer of file reader
        // it may or may not be filled with enough data to decode the record
        // cursor / available bytes MUST be big enough to accomodate at least the Meta-Data
        // since header is what is to decode the rest (iteratively).
        // 1. Meta-Data Reading:
        //      A- Fits in the buffer
        //          - we can decode it directly from the buffer
        //      B- Doesn't fit in the buffer
        //          - we would have to move cursor far enough back to accomdate for the fixed header
        //          - could simply move cursor to the beginning
        //          - then we could meta-data decode after that
        // 2. Record Payload Reading:
        //     A- Required size based on Meta-Data Info fits in the inline bufReader internal buffer
        //         - decode it directly from the bufReader internal buffer without additional copies
        //     B- Required size based on Meta-Data Info DOESN't FIT in the inline bufReader internal buffer
        //         - allocate an out of place buffer big enough to accomdate the record payload size
        //         - decode it in that out of place buffer
        /*
                   ┌─────────────────────────┐
                   │ Fill BufReader Buffer   │
                   └────────────┬────────────┘
                                │
                                ▼
                   ┌─────────────────────────┐
                   │ Enough bytes for fixed  │
                   │ Meta-Data?              │
                   └────────────┬────────────┘
                          YES   │   NO
                       ┌────────┘   └──────────────┐
                       ▼                           ▼
             ┌──────────────────┐      ┌─────────────────────┐
             │ Decode Meta-Data │      │ Move/compact cursor │
             │ from buffer      │      │ to make room        │
             └────────┬─────────┘      └──────────┬──────────┘
                      │                           │
                      │                           ▼
                      │                 ┌────────────────────┐
                      │                 │ Refill buffer      │
                      │                 └──────────┬─────────┘
                      │                            │
                      └──────────────┬─────────────┘
                                     ▼
                           ┌──────────────────────┐
                           │ Determine Payload    │
                           │ Size from Meta-Data  │
                           └──────────┬───────────┘
                                      │
                             ┌────────┴─────────┐
                             │                  │
                            FIT                DOESN'T FIT
                             │                  │
                             ▼                  ▼
                   ┌─────────────────┐   ┌──────────────────┐
                   │ Decode directly │   │ Allocate         │
                   │ from internal   │   │ out-of-place     │
                   │ BufReader       │   │ payload buffer   │
                   │ buffer          │   └────────┬─────────┘
                   └────────┬────────┘            │
                            │                     ▼
                            │             ┌──────────────────┐
                            │             │ Fill payload     │
                            │             │ into allocated   │
                            │             │ buffer           │
                            │             └────────┬─────────┘
                            │                      │
                            └──────────┬───────────┘
                                       ▼
                             ┌────────────────────┐
                             │ Decode Payload     │
                             └─────────┬──────────┘
                                       ▼
                             ┌────────────────────┐
                             │ Record Complete    │
                             └────────────────────┘
        */
        // Sizing:
        //  A. Setting max limits for records sizes
        //      - we could sit a hard limit on max records to avoid large kv entries which could result in huge allocations
        //      - Like tigerbeetle setting hard limits on everything
        //      - but we will keep it simple for now
        //  B. Make all records have same size
        //      - simpler, easier in SerDes
        //      - very good for random reads, requires no indexing
        //      - But wastes Disk-Space and Disk-Bandwidth
        // Copying:
        //  A. Copying Count:
        //      - Copy[1]: from os-page cache to the internal buffers,
        //      - Copy[2]: from internal buffers to consruct ImdbStorageRecord
        //  B. Zero-Copy
        //      - We could use avoid BufReader completly , and use Our own buffers + Direct-IO
        //      - that buffer gets moved instead of being pooled, reused by BufReader
        //      - that buffer is sliced and borrowed from to create the ImdbStorageRecord
        // we wil stick to copying, not setting limit on sizes for now JUST for simplicity

        // A. if data is already cached, we decode directly from it
        //      - this involved 1 Copy only (from internal buffer to created ImdbStorageRecord)
        // B. if data is not cached,
        // NOTE: we could SIMPLIFY this and always use "buf_reader.read_exact(buffer)" but this will cause 2 copies to happen at all times
        // Either from (internal buffer to supplied buffer then from supplied to ImdbStorageRecrod)
        // OR From (Os-Page Cache to supplied buffer then from supplied buffer to ImdbStorageRecord)

        let mut buffer = buf_reader.buffer();
        if buffer.len() < INLINE_STORAGE_RECORD_HEADER_SIZE as usize {
            buffer = buf_reader.fill_buf().map_err(|err: std::io::Error| {
                let formatted_error = format!("[Imdb Error happend while loading record]: {}", err);
                formatted_error
            })?
        }
        // if empty after re-filling then it may be the last record
        if buffer.is_empty() {
            return Result::Ok(None);
        }
        // if only partial Meta-Data entry is read even after re-filling, then it may have been corrupted
        // cause at this point whole INLINE_STORAGE_RECORD_HEADER_SIZE should be in memory
        if buffer.len() < INLINE_STORAGE_RECORD_HEADER_SIZE as usize {
            let formatted_error = format!(
                "[Imdb Error happend while loading Header]: expected {} bytes cached, found {} it may have been corrupted | truncated",
                INLINE_STORAGE_RECORD_HEADER_SIZE,
                buffer.len(),
            );
            return Result::Err(formatted_error);
        }
        let mut slicer = Slicer::new(buffer);
        let checksum_slice = slicer.next_slice(CHECK_SUM_SIZE);
        let (_, checksum_size) = decode_checksum(checksum_slice)?;
        let metadata_buffer_slice = slicer.next_slice(META_DATA_SIZE);
        let (decoded_metadata, metadata_size) = decode_metadata(metadata_buffer_slice)?;
        let required_record_size = decoded_metadata.key_len + decoded_metadata.val_len;

        // advance, pass over checksum, metadata
        let checksum_offset =
            advance_internal_buffer_cursor_by_check_sum(buf_reader, checksum_size)?;
        let metadata_offset =
            advance_internal_buffer_cursor_by_metadata(buf_reader, metadata_size)?;

        // move over buffer to point to the payload slice
        // so that slice is now starting from payload
        // so that slice size reflects availble payload size (not containing header)
        let buffer = buf_reader.buffer();

        let (decoded_record, _record_size, record_offset) =
            if buffer.len() < required_record_size as usize {
                // if payload larger than existing internal buffer (2-Copies branch)
                // we create an out of place buffer that copies from the internal buffer (the cached bytes) + os-page cache
                // BufReader may bypass its own internal buffer (read some from it, some from os-page cache)
                // if not all bytes is available / cached internally
                // bufReader handles
                //  1. Routing bytes copying (some from internal buffer, some from os-page cache)
                //  2. Advancing internal cursors (internal buffer cursor, file cursor)
                let mut record_buffer = vec![b'0'; required_record_size as usize];
                let record_offset = buf_reader.stream_position().map_err(|err| {
                    return format!(
                        "[Imdb Pager error while getting record payload offset]: {}",
                        err
                    );
                })?;
                buf_reader
                    .read_exact(record_buffer.as_mut_slice())
                    .map_err(|_| {
                        let formatted_error = format!("");
                        return formatted_error;
                    })?;
                let (record, record_size) =
                    decode_record_payload(&decoded_metadata, record_buffer.as_slice())?;
                (record, record_size, record_offset)
            } else {
                // if payload fits within internal buffer
                // then we decode directly from that internal buffer (1-Copy branch)
                // we advance cursor manually
                let (record, record_size) = decode_record_payload(&decoded_metadata, buffer)?;
                let record_offset =
                    advance_internal_buffer_cursor_by_record_payload(buf_reader, record_size)?;
                (record, record_size, record_offset)
            };

        let storage_record = ImdbInlineMetaDataStorageReadEntry {
            record: decoded_record,
            record_offset,
            metadata: decoded_metadata,
            metadata_offset,
            identfying_offset: checksum_offset,
        };
        return Result::Ok(Some(storage_record));
    }

    pub fn read_next_record_and_meta_data(
        &mut self,
    ) -> Result<Option<ImdbInlineMetaDataStorageReadEntry>, String> {
        return ImdbInlineMetaDataPager::load_record(&mut self.buf_reader);
    }

    pub fn read_specific_record_and_meta_data(
        &mut self,
        offset: Offset,
    ) -> Result<Option<ImdbInlineMetaDataStorageReadEntry>, String> {
        // this random access always flushes internal buffer
        // so access using this method always involve fetching data from Os-Page-Cache | Disk if not cached
        self.random_access_buf_reader
            .seek(SeekFrom::Start(offset))
            .map_err(|err| {
                return format!("[Imdb Random Seek failure]: {}", err);
            })?;
        return ImdbInlineMetaDataPager::load_record(&mut self.random_access_buf_reader);
    }
}

impl ImdbRecordPager for ImdbInlineMetaDataPager {
    type ReadStorageEntryType = ImdbInlineMetaDataStorageReadEntry;

    fn load_next_record_and_metadata(
        &mut self,
    ) -> Result<Option<Self::ReadStorageEntryType>, String> {
        return self.read_next_record_and_meta_data();
    }

    fn load_specific_record_and_meta_data_using_id_offset(
        &mut self,
        offset: Offset,
    ) -> Result<Option<Self::ReadStorageEntryType>, String> {
        return self.read_specific_record_and_meta_data(offset);
    }
}

impl Iterator for ImdbInlineMetaDataPager {
    type Item = Result<ImdbInlineMetaDataStorageReadEntry, String>;
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
        mocking_utils::{
            inline_metadata_mocking_utils::find_record_offset,
            records_paging::{verify_record, write_mock_records, write_records},
        },
        record::imdb_record::{ImdbRecord, ImdbRecordMetaData},
        storage::{
            imdb_inline_metadata_storage_engine::{
                imdb_inline_metadata_format::encode_record,
                imdb_inline_metadata_pager::ImdbInlineMetaDataPager,
                imdb_inline_metadata_storage_entries::ImdbInlineMetaDataStorageReadEntry,
                imdb_inline_metadata_storage_record::{
                    CHECK_SUM_SIZE, INLINE_STORAGE_RECORD_HEADER_SIZE,
                    ImdbInlineMetaDataStorageRecord,
                },
            },
            pager::ImdbRecordPager,
        },
    };

    fn create_test_dir_and_test_file(suffix: &str) -> String {
        let dir_path = Path::new("inline_metadata_pager_unit_tests");
        let mut dir_path = dir_path.to_path_buf();
        dir_path.push(suffix);
        std::fs::create_dir_all(&dir_path).unwrap();
        dir_path.add_extension(".bin");
        let path: String = dir_path.to_str().to_owned().unwrap().to_string();
        return path;
    }

    #[test]
    pub fn test_loading_records() {
        let file_path = create_test_dir_and_test_file("loading_test");
        let file_path = Path::new(&file_path);
        let records_count = 100;
        write_mock_records(file_path, records_count, &mut encode_record);

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
        let file_path = create_test_dir_and_test_file("iterator");
        let file_path = Path::new(&file_path);
        let records_count = 100;
        write_mock_records(file_path, records_count, &mut encode_record);

        let pager = ImdbInlineMetaDataPager::new(file_path).unwrap();

        for (i, result) in pager.enumerate() {
            let entry = result.unwrap();
            assert!(verify_record(i, &entry.record));
        }
    }

    pub fn verify_inline_metadata_fetched_storage_record_offsets(
        fetched_storage_record: Option<ImdbInlineMetaDataStorageReadEntry>,
        original_records: &Vec<ImdbInlineMetaDataStorageRecord>,
        record_to_verify_against_index: usize,
    ) {
        assert!(fetched_storage_record.is_some());
        let fetched_storage_record = fetched_storage_record.unwrap();
        let (starting_offset, record_offset_expected) =
            find_record_offset(original_records, record_to_verify_against_index);

        assert_eq!(fetched_storage_record.identfying_offset, starting_offset);
        assert_eq!(
            fetched_storage_record.metadata_offset,
            starting_offset + CHECK_SUM_SIZE as u64
        );
        assert_eq!(
            fetched_storage_record.record_offset,
            record_offset_expected as u64
        );
        assert_eq!(
            fetched_storage_record.metadata.key_len,
            original_records[record_to_verify_against_index]
                .metadata
                .key_len
        );
        assert_eq!(
            fetched_storage_record.metadata.val_len,
            original_records[record_to_verify_against_index]
                .metadata
                .val_len
        );
        assert_eq!(
            fetched_storage_record.record.key,
            original_records[record_to_verify_against_index].record.key
        );
        assert_eq!(
            fetched_storage_record.record.value,
            original_records[record_to_verify_against_index]
                .record
                .value
        );
    }

    #[test]
    pub fn test_random_read() {
        let mut records = Vec::new();
        records.push(ImdbInlineMetaDataStorageRecord {
            record: ImdbRecord {
                key: "1".as_bytes().to_vec(),
                value: "name".as_bytes().to_vec(),
            },
            metadata: ImdbRecordMetaData {
                key_len: 1,
                val_len: 4,
            },
            check_sum: 0,
        });
        records.push(ImdbInlineMetaDataStorageRecord {
            record: ImdbRecord {
                key: "123".as_bytes().to_vec(),
                value: "another_name".as_bytes().to_vec(),
            },
            metadata: ImdbRecordMetaData {
                key_len: 3,
                val_len: 12,
            },
            check_sum: 0,
        });
        records.push(ImdbInlineMetaDataStorageRecord {
            record: ImdbRecord {
                key: "123791237981273".as_bytes().to_vec(),
                value: "mashekjwhrkjwehrkjw".as_bytes().to_vec(),
            },
            metadata: ImdbRecordMetaData {
                key_len: 15,
                val_len: 19,
            },
            check_sum: 0,
        });
        let path = create_test_dir_and_test_file("random_read");
        let path = Path::new(&path);
        write_records(&records, path, &mut encode_record);

        let mut pager = ImdbInlineMetaDataPager::new(&path).unwrap();

        for (i, _) in records.iter().enumerate() {
            let (metadata_offset, _) = find_record_offset(&records, i);
            let storage_record = pager
                .load_specific_record_and_meta_data_using_id_offset(metadata_offset)
                .unwrap();
            verify_inline_metadata_fetched_storage_record_offsets(storage_record, &records, i);
        }
    }

    #[test]
    pub fn test_iterative_read_records_payload_spanning_multiple_buffer_reads() {
        let path = create_test_dir_and_test_file("spanning_multiple_buffers_reads");
        let path = Path::new(&path);

        let mut pager = ImdbInlineMetaDataPager::new(path).unwrap();
        let buffer_capacity = pager.buf_reader.capacity();

        let mut records = Vec::new();
        let key = vec![b'1'; buffer_capacity];
        let value = vec![b'2'; buffer_capacity];
        records.push(ImdbInlineMetaDataStorageRecord {
            record: ImdbRecord {
                key: key.clone(),
                value: value.clone(),
            },
            metadata: ImdbRecordMetaData {
                key_len: key.len() as u32,
                val_len: value.len() as u32,
            },
            check_sum: 0,
        });

        write_records(&records, path, &mut encode_record);

        let storage_record = pager.load_next_record_and_metadata().unwrap();
        assert!(storage_record.is_some());
        let storage_record = storage_record.unwrap();

        // verify id
        assert_eq!(storage_record.identfying_offset, 0);

        // verify metadata
        assert_eq!(storage_record.metadata.key_len, records[0].metadata.key_len);
        assert_eq!(storage_record.metadata.val_len, records[0].metadata.val_len);

        // verify record
        assert_eq!(storage_record.record.key, records[0].record.key);
        assert_eq!(storage_record.record.value, records[0].record.value);
    }

    #[test]
    pub fn test_iterative_read_metadata_not_fitting_in_buffer() {
        let path = create_test_dir_and_test_file("header_spanning_multiple_buffers_reads");
        let path = Path::new(&path);
        let pager = ImdbInlineMetaDataPager::new(path).unwrap();
        let buffer_capacity = pager.buf_reader.capacity();

        let mut records = Vec::new();
        let key_size = 100;
        let key = vec![b'1'; key_size];
        // takes the rest of the buffer, expect 10 bytes to leave room for next header of next record
        // so next record's header could span current buffer + and another one
        let value =
            vec![b'2'; buffer_capacity - key_size - INLINE_STORAGE_RECORD_HEADER_SIZE as usize];
        records.push(ImdbInlineMetaDataStorageRecord {
            record: ImdbRecord {
                key: key.clone(),
                value: value.clone(),
            },
            metadata: ImdbRecordMetaData {
                key_len: key.len() as u32,
                val_len: value.len() as u32,
            },
            check_sum: 0,
        });
        records.push(ImdbInlineMetaDataStorageRecord {
            record: ImdbRecord {
                key: value.clone(),
                value: key.clone(),
            },
            metadata: ImdbRecordMetaData {
                key_len: value.len() as u32,
                val_len: key.len() as u32,
            },
            check_sum: 0,
        });

        write_records(&records, path, &mut encode_record);

        for (i, record) in pager.enumerate() {
            let (starting_offset, record_offset) = find_record_offset(&records, i);
            let storage_record = record.unwrap();

            // offsets
            assert_eq!(storage_record.identfying_offset, starting_offset);
            assert_eq!(storage_record.record_offset, record_offset);
            assert_eq!(
                storage_record.metadata_offset,
                starting_offset + CHECK_SUM_SIZE as u64
            );

            // verify metadata
            assert_eq!(storage_record.metadata.key_len, records[i].metadata.key_len);
            assert_eq!(storage_record.metadata.val_len, records[i].metadata.val_len);

            // verify record
            assert_eq!(storage_record.record.key, records[i].record.key);
            assert_eq!(storage_record.record.value, records[i].record.value);
        }
    }

    #[test]
    pub fn test_random_read_records_payload_spanning_multiple_buffers() {
        let path = create_test_dir_and_test_file("andom_reads_spanning_multiple_buffers_reads");
        let path = Path::new(&path);

        let mut pager = ImdbInlineMetaDataPager::new(path).unwrap();
        let buffer_capacity = pager.buf_reader.capacity();

        let mut records = Vec::new();
        let key_size = 100;
        let key = vec![b'1'; key_size];
        // takes the rest of the buffer, expect 10 bytes to leave room for next header of next record
        // so next record's header could span current buffer + and another one
        let value =
            vec![b'2'; buffer_capacity - key_size - INLINE_STORAGE_RECORD_HEADER_SIZE as usize];
        records.push(ImdbInlineMetaDataStorageRecord {
            record: ImdbRecord {
                key: key.clone(),
                value: value.clone(),
            },
            metadata: ImdbRecordMetaData {
                key_len: key.len() as u32,
                val_len: value.len() as u32,
            },
            check_sum: 0,
        });
        let storage_record = ImdbInlineMetaDataStorageRecord {
            record: ImdbRecord {
                key: key.clone(),
                value: value.clone(),
            },
            metadata: ImdbRecordMetaData {
                key_len: key.len() as u32,
                val_len: value.len() as u32,
            },
            check_sum: 0,
        };
        records.push(storage_record);

        write_records(&records, path, &mut encode_record);

        for i in 0..records.len() {
            let (start_offset, record_offset) = find_record_offset(&records, i);
            let storage_record = pager
                .load_specific_record_and_meta_data_using_id_offset(start_offset)
                .unwrap()
                .unwrap();

            // offsets
            assert_eq!(storage_record.identfying_offset, start_offset);
            assert_eq!(storage_record.record_offset, record_offset);
            assert_eq!(
                storage_record.metadata_offset,
                start_offset + CHECK_SUM_SIZE as u64
            );

            // verify metadata
            assert_eq!(storage_record.metadata.key_len, records[i].metadata.key_len);
            assert_eq!(storage_record.metadata.val_len, records[i].metadata.val_len);

            // verify record
            assert_eq!(storage_record.record.key, records[i].record.key);
            assert_eq!(storage_record.record.value, records[i].record.value);
        }
    }

    #[test]
    pub fn test_random_read_at_wrong_offset_at_file_end() {
        let path = create_test_dir_and_test_file("random_read_at_wron_metadata_offse_at_file_end");
        let path = Path::new(&path);

        write_mock_records(path, 1, &mut encode_record);
        let mut pager = ImdbInlineMetaDataPager::new(path).unwrap();
        let res = pager.load_specific_record_and_meta_data_using_id_offset(1);
        assert!(res.is_err());
    }
}
