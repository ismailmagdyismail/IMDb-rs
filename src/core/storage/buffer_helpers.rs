use std::{
    fs::File,
    io::{BufRead, BufReader, Seek},
};

pub fn advance_internal_buffer_cursor(
    buffer: &mut BufReader<File>,
    metadata_size: u32,
    record_size: u32,
) -> Result<(u64, u64), String> {
    // record offset of metadata
    // slide the window over by header
    let metadata_offset = buffer.stream_position().map_err(|err| {
        return format!("[Imdb Pager error while getting metadata offset]: {}", err);
    })?;
    // slider cursor over to consume meta-data
    buffer.consume(metadata_size as usize);

    // record offset of record
    // slide the window again to pass record
    let record_offset = buffer.stream_position().map_err(|err| {
        return format!("[Imdb Pager error while getting record offset]: {}", err);
    })?;
    buffer.consume(record_size as usize);

    Ok((metadata_offset, record_offset))
}
