/*
// ************************ 1. Record Format ************************************

- might later remove key_value len from here, keep it in the index file
- to allow index access to know bytes needed to be fetched from payload
- resulting in more efficient , batchable disk access, less round trips
+---------------+--------------+----------------+------------+---------------+
| checksum[u32] | key_len[u32] | value_len[u32] | key_payload| value_payload |
+---------------+--------------+----------------+------------+---------------+
*/

use crate::core::{
    checksum::check_sum::CheckSum,
    record::imdb_record::{ImdbRecord, ImdbRecordMetaData, META_DATA_SIZE},
    serdes::slicer::Slicer,
    storage::{
        imdb_inline_metadata_storage_engine::imdb_inline_metadata_storage_record::{
            CHECK_SUM_SIZE, INLINE_STORAGE_RECORD_HEADER_SIZE, ImdbInlineMetaDataStorageRecord,
        },
        imdb_storage_operations_status::ImdbStorageError,
    },
};

// decodes only meta data
// buffer supplied must have a big enough size to accomodate MetaData
// could be used for iterative decoding (first header, then allocating large enough buffer for payload)
pub fn decode_metadata(buffer: &[u8]) -> Result<(ImdbRecordMetaData, u32), ImdbStorageError> {
    let required_buffer_size = META_DATA_SIZE;
    if buffer.len() < required_buffer_size as usize {
        return Result::Err(ImdbStorageError::DecodeInSufficientBufferSize("metadata"));
    }
    let (metadata, metadata_size) = ImdbRecordMetaData::deserialize_copy(buffer)?;
    return Result::Ok((metadata, metadata_size));
}

// decodes payload using supplied meta data info
// buffer supplied must have a big enough size to accomodate record's Payload
// could be used for iterative decoding (first header, then allocating large enough buffer for payload)
pub fn decode_record_payload(
    metadata: &ImdbRecordMetaData,
    buffer: &[u8],
) -> Result<(ImdbRecord, u32), ImdbStorageError> {
    let required_buffer_size = metadata.key_len + metadata.val_len;
    if buffer.len() < required_buffer_size as usize {
        return Result::Err(ImdbStorageError::DecodeInSufficientBufferSize(
            "record_payload",
        ));
    }
    let (record, record_size) = ImdbRecord::deserialize_copy(&metadata, buffer)?;
    Result::Ok((record, record_size))
}

pub fn decode_checksum(buffer: &[u8]) -> Result<(u32, u32), ImdbStorageError> {
    let required_buffer_size = CHECK_SUM_SIZE;
    if buffer.len() < required_buffer_size as usize {
        return Result::Err(ImdbStorageError::DecodeInSufficientBufferSize("checksum"));
    }
    let checksum_buffer: Vec<u8> = Vec::from(&buffer[0..CHECK_SUM_SIZE as usize]);
    let checksum = u32::from_le_bytes(checksum_buffer.try_into().unwrap());
    Ok((checksum, CHECK_SUM_SIZE))
}

// buffer supplied must have a big enough size to accomodate MetaData + Record
pub fn decode_whole_record(
    buffer: &[u8],
) -> Result<(ImdbInlineMetaDataStorageRecord, u32), ImdbStorageError> {
    debug_assert!(buffer.len() >= INLINE_STORAGE_RECORD_HEADER_SIZE as usize);

    let mut slicer = Slicer::new(buffer);

    let crc_slice = slicer.next_slice(CHECK_SUM_SIZE);
    let (crc, crc_size) = decode_checksum(crc_slice)?;

    let metadata_slice = slicer.next_slice(META_DATA_SIZE);
    let (metadata, metadata_size) = decode_metadata(metadata_slice)?;

    let payload_slice = slicer.next_slice(metadata.key_len + metadata.val_len);
    let (record, record_size) = decode_record_payload(&metadata, payload_slice)?;

    debug_assert!(metadata_size + crc_size == INLINE_STORAGE_RECORD_HEADER_SIZE);

    let storage_record = ImdbInlineMetaDataStorageRecord {
        record,
        metadata,
        check_sum: crc,
    };
    Result::Ok((storage_record, crc_size + metadata_size + record_size))
}

pub fn encode_record<T>(
    record: &ImdbRecord,
    meta_data: &ImdbRecordMetaData,
    buffer: &mut [u8],
    checksum_calculator: &T,
) -> Result<(), ImdbStorageError>
where
    T: CheckSum,
{
    if buffer.len() < record.ser_size() as usize + INLINE_STORAGE_RECORD_HEADER_SIZE as usize {
        return Result::Err(ImdbStorageError::EncodeInSufficientBufferSize(
            "record_payload",
        ));
    }

    let mut slicer = Slicer::new(buffer);

    // skip the crc slice
    slicer.advance(CHECK_SUM_SIZE);

    // encode meta data
    let metadata_buffer = slicer.next_slice_mut(META_DATA_SIZE);
    let encoded_metdata_size = meta_data.serialize(metadata_buffer)?;

    // encode record payloads
    let record_buffer = slicer.next_slice_mut(meta_data.key_len + meta_data.val_len);
    let encoded_record_size = record.serialize(record_buffer)?;

    // rest of the buffer slices to calc checksum on
    slicer.reset();
    slicer.advance(CHECK_SUM_SIZE);

    // calculate, encode checksum / crc
    let rest = slicer.next_slice_mut(META_DATA_SIZE + meta_data.key_len + meta_data.val_len);
    let checksum_val = checksum_calculator.calculate(rest);

    slicer.reset();
    let crc_buffer = slicer.next_slice_mut(CHECK_SUM_SIZE);
    crc_buffer.copy_from_slice(&checksum_val.to_le_bytes());

    debug_assert!(encoded_metdata_size == META_DATA_SIZE);
    debug_assert!(encoded_record_size == meta_data.key_len + meta_data.val_len);

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::core::{
        checksum::{check_sum::CheckSum, crc32::Crc32CheckSum},
        record::imdb_record::{ImdbRecord, ImdbRecordMetaData},
        storage::imdb_inline_metadata_storage_engine::{
            imdb_inline_metadata_format::{decode_checksum, decode_whole_record, encode_record},
            imdb_inline_metadata_storage_record::{
                CHECK_SUM_SIZE, INLINE_STORAGE_RECORD_HEADER_SIZE,
            },
        },
    };

    #[test]
    fn test_record_serdes() {
        let mut records = Vec::new();
        let mut encoded_records = Vec::new();
        let mut buffers = vec![Vec::<u8>::new(); 100];
        for i in 0..100 {
            let key = i.to_string().as_bytes().to_vec();
            let value = "name".as_bytes().to_vec();
            let key_len = key.len() as u32;
            let val_len = value.len() as u32;
            let record = ImdbRecord { key, value };
            let meta_data = ImdbRecordMetaData { key_len, val_len };
            buffers[i].resize(
                (INLINE_STORAGE_RECORD_HEADER_SIZE + key_len + val_len) as usize,
                b'0',
            );
            let buffer = buffers[i].as_mut_slice();
            let crc_calc = Crc32CheckSum::new();
            encoded_records.push(encode_record(&record, &meta_data, buffer, &crc_calc));
            records.push((meta_data, record));
        }

        for i in 0..100 {
            let (storage_record, _) = decode_whole_record(&buffers[i]).unwrap();
            let actual_meta_data = &records[i].0;
            assert_eq!(storage_record.metadata.key_len, actual_meta_data.key_len);
            assert_eq!(storage_record.metadata.val_len, actual_meta_data.val_len);

            let actual_record = &records[i].1;
            assert_eq!(storage_record.record.key, actual_record.key);
            assert_eq!(storage_record.record.value, actual_record.value);
        }
    }

    #[test]
    fn test_checksum_serdes() {
        let mut records = Vec::new();
        let mut encoded_records = Vec::new();
        let mut buffers = vec![Vec::<u8>::new(); 100];
        for i in 0..100 {
            let key = i.to_string().as_bytes().to_vec();
            let value = "name".as_bytes().to_vec();
            let key_len = key.len() as u32;
            let val_len = value.len() as u32;
            let record = ImdbRecord { key, value };
            let meta_data = ImdbRecordMetaData { key_len, val_len };
            buffers[i].resize(
                (INLINE_STORAGE_RECORD_HEADER_SIZE + key_len + val_len) as usize,
                b'0',
            );
            let buffer = buffers[i].as_mut_slice();
            let crc_calc = Crc32CheckSum::new();
            encoded_records.push(encode_record(&record, &meta_data, buffer, &crc_calc));
            let encoded_checksum = crc_calc.calculate(&buffer[CHECK_SUM_SIZE as usize..]);
            dbg!(encoded_checksum.to_le_bytes());
            let (decoded_checksum, _) = decode_checksum(buffer).unwrap();
            assert_eq!(encoded_checksum, decoded_checksum);
            records.push((meta_data, record));
        }
    }
}
