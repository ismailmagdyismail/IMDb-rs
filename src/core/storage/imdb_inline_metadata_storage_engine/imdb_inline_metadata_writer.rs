use std::{
    fs::{File, OpenOptions},
    io::{Seek, Write},
    path::Path,
};

use crate::core::{
    record::imdb_record::{ImdbRecord, ImdbRecordMetaData},
    storage::{
        imdb_inline_metadata_storage_engine::imdb_inline_metadata_format::encode_record,
        writer::{ImdbRecordWriter, ImdbStorageEntry},
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

    pub fn write_record(
        &mut self,
        metadata: &ImdbRecordMetaData,
        record: &ImdbRecord,
    ) -> Result<ImdbStorageEntry, String> {
        let mut buffer = Vec::new();
        buffer.resize(
            record.ser_size() as usize + metadata.ser_size() as usize,
            b'0',
        );
        encode_record(record, metadata, &mut buffer)?;
        let metadata_offset = self.writer.stream_position().map_err(|err| {
            let fmt_error = format!("[Imdb Writer Error happened while writing record]: {}", err);
            return fmt_error;
        })?;
        let record_offset = metadata_offset + metadata.ser_size() as u64;
        self.writer.write_all(&buffer).map_err(|err| {
            let fmt_error = format!("[Imdb Writer Error happened while writing record]: {}", err);
            return fmt_error;
        })?;
        let storage_entry = ImdbStorageEntry {
            record_offset,
            metadata_offset,
            identfying_offset: metadata_offset,
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
    fn write_record_and_metadata(
        &mut self,
        metadata: &ImdbRecordMetaData,
        record: &ImdbRecord,
    ) -> Result<ImdbStorageEntry, String> {
        self.write_record(metadata, record)
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
        record::imdb_record::{ImdbRecord, ImdbRecordMetaData},
        storage::{
            imdb_inline_metadata_storage_engine::imdb_inline_metadata_writer::ImdbInlineMetaDataWriter,
            writer::ImdbRecordWriter,
        },
    };

    #[test]
    fn test_writing_basic_record() {
        let key = "1".as_bytes().to_owned();
        let value = "val".as_bytes().to_owned();

        let metadata = ImdbRecordMetaData {
            key_len: key.len() as u32,
            val_len: value.len() as u32,
            check_sum: 0,
        };
        let record = ImdbRecord {
            key: key,
            value: value,
        };

        let file_path = "inline_metadata_basic_records_writer.bin";
        create_writer_file(file_path);
        let mut writer = ImdbInlineMetaDataWriter::new(Path::new(file_path)).unwrap();
        let storage_entry = writer
            .write_record_and_metadata(&metadata, &record)
            .unwrap();

        assert_eq!(0u64, storage_entry.identfying_offset);
        assert_eq!(0, storage_entry.metadata_offset);
        assert_eq!(0 + metadata.ser_size() as u64, storage_entry.record_offset);
    }

    #[test]
    fn test_writing_multiple_records() {
        let file_path = "inline_metadata_multiple_records_writer.bin";
        create_writer_file(file_path);
        let mut writer = ImdbInlineMetaDataWriter::new(Path::new(file_path)).unwrap();
        let records = create_records(100);
        for (i, (record, metadata)) in records.iter().enumerate() {
            let storage_entry = writer.write_record_and_metadata(metadata, record).unwrap();
            let (metadata_offset, record_offset) = find_record_offset(&records, i);
            assert_eq!(storage_entry.identfying_offset, metadata_offset);
            assert_eq!(storage_entry.metadata_offset, metadata_offset);
            assert_eq!(storage_entry.record_offset, record_offset);
        }
    }

    #[test]
    fn test_writing_in_already_populated_file() {
        let file_path = "inline_metadata_already_populated_file_writer.bin";
        create_writer_file(file_path);

        let mut last_offset = 0;
        {
            let mut writer = ImdbInlineMetaDataWriter::new(Path::new(file_path)).unwrap();
            let records = create_records(100);
            for (i, (record, metadata)) in records.iter().enumerate() {
                let storage_entry = writer.write_record_and_metadata(metadata, record).unwrap();
                let (metadata_offset, record_offset) = find_record_offset(&records, i);
                assert_eq!(storage_entry.identfying_offset, metadata_offset);
                assert_eq!(storage_entry.metadata_offset, metadata_offset);
                assert_eq!(storage_entry.record_offset, record_offset);
                last_offset = record_offset + record.ser_size() as u64;
            }
        }

        {
            let mut writer = ImdbInlineMetaDataWriter::new(Path::new(file_path)).unwrap();
            let records = create_records(100);
            for (i, (record, metadata)) in records.iter().enumerate() {
                let storage_entry = writer.write_record_and_metadata(metadata, record).unwrap();
                let (metadata_offset, record_offset) =
                    find_record_offset_with_starting_offset(&records, i, last_offset);
                assert_eq!(storage_entry.identfying_offset, metadata_offset);
                assert_eq!(storage_entry.metadata_offset, metadata_offset);
                assert_eq!(storage_entry.record_offset, record_offset);
            }
        }
    }
}
