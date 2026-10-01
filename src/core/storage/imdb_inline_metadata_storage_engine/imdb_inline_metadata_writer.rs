use std::{
    fs::{File, OpenOptions},
    io::{Seek, Write},
    path::Path,
};

use crate::core::{
    record::imdb_record::{ImdbRecord, ImdbRecordMetaData}, storage::{
        imdb_inline_metadata_storage_engine::{
            imdb_inline_metadata_format::encode_record,
            imdb_inline_metadata_storage_record::{
                CHECK_SUM_SIZE, INLINE_STORAGE_RECORD_HEADER_SIZE,
            },
        }, imdb_storage_entries::ImdbStorageWriteEntry, writer::ImdbRecordWriter,
    },
};

pub struct ImdbInlineMetaDataWriter {
    writer: File,
}

impl ImdbInlineMetaDataWriter {
    pub fn new(file_path: &Path) -> Result<ImdbInlineMetaDataWriter, String> {
        let mut options = OpenOptions::new();
        let mut file = options
            .write(true)
            .append(true)
            .open(file_path)
            .map_err(|err| {
                let formatted_record = format!("[Imdb File Writer error]: {}", err);
                return formatted_record;
            })?;
        file.seek(std::io::SeekFrom::End(0)).map_err(|err| {
            let fmt_error = format!(
                "[Imdb Writer Error]: couldn't seek to the end of the file {}",
                err
            );
            fmt_error
        })?;
        let writer = ImdbInlineMetaDataWriter { writer: file };
        return Ok(writer);
    }

    pub fn write_record(&mut self, record: &ImdbRecord) -> Result<ImdbStorageWriteEntry, String> {
        let metadata = ImdbRecordMetaData::from(record);
        let mut buffer = Vec::new();
        buffer.resize(
            record.ser_size() as usize + INLINE_STORAGE_RECORD_HEADER_SIZE as usize,
            b'0',
        );
        encode_record(record, &metadata, &mut buffer)?;
        let starting_offset = self.writer.stream_position().map_err(|err| {
            let fmt_error = format!("[Imdb Writer Error happened while writing record]: {}", err);
            return fmt_error;
        })?;
        let metadata_offset = starting_offset + CHECK_SUM_SIZE as u64;
        let record_offset = starting_offset + INLINE_STORAGE_RECORD_HEADER_SIZE as u64;
        self.writer.write_all(&buffer).map_err(|err| {
            let fmt_error = format!("[Imdb Writer Error happened while writing record]: {}", err);
            return fmt_error;
        })?;
        let storage_entry = ImdbStorageWriteEntry {
            record_offset,
            metadata_offset,
            identfying_offset: starting_offset,
        };
        Ok(storage_entry)
    }

    // flush content to os-page cache
    // fsync to sync os-page cache pages with disk
    // suffers from "fsync-gate" problem (since we are not using direct-IO)
    pub fn flush_and_fsync(&mut self) -> Result<(), String> {
        self.writer.flush().map_err(|err| {
            let fmt_error = format!(
                "[Imdb Writer Flushing Error happened while flushing record]: {}",
                err
            );
            return fmt_error;
        })?;
        self.writer.sync_all().map_err(|err| {
            let fmt_error = format!("[Imdb Writer fsync Error happened]: {}", err);
            return fmt_error;
        })?;
        Ok(())
    }
}

impl ImdbRecordWriter for ImdbInlineMetaDataWriter {
    fn append_record(&mut self, record: &ImdbRecord) -> Result<ImdbStorageWriteEntry, String> {
        self.write_record(record)
    }

    fn sync(&mut self) -> Result<(), String> {
        self.flush_and_fsync()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::core::{
        mocking_utils::{
            inline_metadata_mocking_utils::{
                find_record_offset, find_record_offset_with_starting_offset,
            },
            records_paging::{create_records, create_writer_file},
        },
        record::imdb_record::ImdbRecord,
        storage::{
            imdb_inline_metadata_storage_engine::{
                imdb_inline_metadata_storage_record::{
                    CHECK_SUM_SIZE, INLINE_STORAGE_RECORD_HEADER_SIZE,
                },
                imdb_inline_metadata_writer::ImdbInlineMetaDataWriter,
            },
            writer::ImdbRecordWriter,
        },
    };

    fn create_test_dir_and_test_file(suffix: &str) -> String {
        let dir_path = Path::new("inline_metadata_writer_unit_tests");
        let mut dir_path = dir_path.to_path_buf();
        dir_path.push(suffix);
        std::fs::create_dir_all(&dir_path).unwrap();
        dir_path.add_extension(".bin");
        let path: String = dir_path.to_str().to_owned().unwrap().to_string();
        return path;
    }

    #[test]
    fn test_writing_basic_record() {
        let key = "1".as_bytes().to_owned();
        let value = "val".as_bytes().to_owned();

        let record = ImdbRecord {
            key: key,
            value: value,
        };

        let file_path = create_test_dir_and_test_file("inline_metadata_basic_records_writer");
        create_writer_file(file_path.as_str());
        let mut writer = ImdbInlineMetaDataWriter::new(Path::new(&file_path)).unwrap();
        let storage_entry = writer.append_record(&record).unwrap();

        assert_eq!(0u64, storage_entry.identfying_offset);
        assert_eq!(0 + CHECK_SUM_SIZE as u64, storage_entry.metadata_offset);
        assert_eq!(
            INLINE_STORAGE_RECORD_HEADER_SIZE as u64,
            storage_entry.record_offset
        );
    }

    #[test]
    fn test_writing_multiple_records() {
        let file_path = create_test_dir_and_test_file("inline_metadata_multiple_records_writer");
        create_writer_file(file_path.as_str());
        let mut writer = ImdbInlineMetaDataWriter::new(Path::new(&file_path)).unwrap();
        let records = create_records(100);
        for (i, storage_record) in records.iter().enumerate() {
            let storage_entry = writer.append_record(&storage_record.record).unwrap();
            let (start_offset, record_offset) = find_record_offset(&records, i);
            assert_eq!(storage_entry.identfying_offset, start_offset);
            assert_eq!(
                storage_entry.metadata_offset,
                start_offset + CHECK_SUM_SIZE as u64
            );
            assert_eq!(storage_entry.record_offset, record_offset);
        }
    }

    #[test]
    fn test_writing_in_already_populated_file() {
        let file_path =
            create_test_dir_and_test_file("inline_metadata_already_populated_file_writer");
        create_writer_file(file_path.as_str());

        let mut last_offset = 0;
        {
            let mut writer = ImdbInlineMetaDataWriter::new(Path::new(&file_path)).unwrap();
            let records = create_records(100);
            for (i, storage_record) in records.iter().enumerate() {
                let storage_entry = writer.append_record(&storage_record.record).unwrap();
                let (starting_offset, record_offset) = find_record_offset(&records, i);
                assert_eq!(storage_entry.identfying_offset, starting_offset);
                assert_eq!(
                    storage_entry.metadata_offset,
                    starting_offset + CHECK_SUM_SIZE as u64
                );
                assert_eq!(storage_entry.record_offset, record_offset);
                last_offset = record_offset + storage_record.record.ser_size() as u64;
            }
        }

        {
            let mut writer = ImdbInlineMetaDataWriter::new(Path::new(&file_path)).unwrap();
            let records = create_records(100);
            for (i, storage_record) in records.iter().enumerate() {
                let storage_entry = writer.append_record(&storage_record.record).unwrap();
                let (starting_offset, record_offset) =
                    find_record_offset_with_starting_offset(&records, i, last_offset);
                assert_eq!(storage_entry.identfying_offset, starting_offset);
                assert_eq!(
                    storage_entry.metadata_offset,
                    starting_offset + CHECK_SUM_SIZE as u64
                );
                assert_eq!(storage_entry.record_offset, record_offset);
            }
        }
    }
}
