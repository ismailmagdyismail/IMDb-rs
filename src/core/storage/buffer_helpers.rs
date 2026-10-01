use std::{
    fs::File,
    io::{BufRead, BufReader, Seek},
};

use crate::core::storage::imdb_storage_operations_status::ImdbStorageError;

pub fn advance_internal_buffer_cursor_by_check_sum(
    buffer: &mut BufReader<File>,
    checksum_size: u32,
) -> Result<u64, ImdbStorageError> {
    // record offset of metadata
    // slide the window over by header
    let checksum_offset = buffer
        .stream_position()
        .map_err(|err| ImdbStorageError::DiskSeek("checksum", err.to_string()))?;
    // slider cursor over to consume meta-data
    buffer.consume(checksum_size as usize);
    return Ok(checksum_offset);
}

pub fn advance_internal_buffer_cursor_by_metadata(
    buffer: &mut BufReader<File>,
    metadata_size: u32,
) -> Result<u64, ImdbStorageError> {
    // record offset of metadata
    // slide the window over by header
    let metadata_offset = buffer
        .stream_position()
        .map_err(|err| ImdbStorageError::DiskSeek("metadata", err.to_string()))?;
    // slider cursor over to consume meta-data
    buffer.consume(metadata_size as usize);
    return Ok(metadata_offset);
}

pub fn advance_internal_buffer_cursor_by_record_payload(
    buffer: &mut BufReader<File>,
    record_size: u32,
) -> Result<u64, ImdbStorageError> {
    // record offset of record
    // slide the window again to pass record
    let record_offset = buffer
        .stream_position()
        .map_err(|err| ImdbStorageError::DiskSeek("record_payload", err.to_string()))?;
    buffer.consume(record_size as usize);

    Ok(record_offset)
}

pub fn advance_internal_buffer_cursor(
    buffer: &mut BufReader<File>,
    metadata_size: u32,
    record_size: u32,
) -> Result<(u64, u64), ImdbStorageError> {
    // record offset of metadata
    // slide the window over by header
    let metadata_offset = advance_internal_buffer_cursor_by_metadata(buffer, metadata_size)?;

    // record offset of record
    // slide the window again to pass record
    let record_offset = advance_internal_buffer_cursor_by_record_payload(buffer, record_size)?;

    Ok((metadata_offset, record_offset))
}
