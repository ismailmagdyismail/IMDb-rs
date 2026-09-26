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
    record::imdb_record::{HEADER_SIZE, ImdbRecord, ImdbRecordMetaData},
    serdes::slicer::Slicer,
};

pub fn decode_record(buffer: &[u8]) -> Result<(ImdbRecordMetaData, ImdbRecord, u32), String> {
    let mut required_buffer_size = HEADER_SIZE;
    if buffer.len() < required_buffer_size as usize {
        let error = format!(
            "[Imdb Decode Record Error]: buffer supplied is smaller than required header size {}, {}",
            required_buffer_size,
            buffer.len()
        );
        return Result::Err(error);
    }
    let mut slicer = Slicer::new(buffer);
    let metadata_buffer = slicer.next_slice(HEADER_SIZE);
    let (metadata, metadata_size) = ImdbRecordMetaData::deserialize_copy(metadata_buffer)?;
    required_buffer_size += metadata.key_len + metadata.val_len;
    if buffer.len() < required_buffer_size as usize {
        let error = format!(
            "[Imdb Decode Record Error]: buffer supplied is smaller than payload size {}, {}",
            required_buffer_size,
            buffer.len(),
        );
        return Result::Err(error);
    }
    let record_buffer = slicer.next_slice(metadata.key_len + metadata.val_len);
    let (record, record_size) = ImdbRecord::deserialize_copy(&metadata, record_buffer)?;
    debug_assert!(metadata_size == HEADER_SIZE);
    debug_assert!(record_size == metadata.key_len + metadata.val_len);
    Ok((metadata, record, record_size + metadata_size))
}

pub fn encode_record(
    record: &ImdbRecord,
    meta_data: &ImdbRecordMetaData,
    buffer: &mut [u8],
) -> Result<(), String> {
    if buffer.len() < record.ser_size() as usize + meta_data.ser_size() as usize {
        let error = format!(
            "[Imdb Encode Record Error]: buffer supplied is smaller than payload size {}, {}",
            record.ser_size() + meta_data.ser_size(),
            buffer.len(),
        );
        return Result::Err(error);
    }

    let mut slicer = Slicer::new(buffer);

    let metadata_buffer = slicer.next_slice_mut(HEADER_SIZE);
    let encoded_metdata_size = meta_data.serialize(metadata_buffer)?;

    let record_buffer = slicer.next_slice_mut(meta_data.key_len + meta_data.val_len);
    let encoded_record_size = record.serialize(record_buffer)?;

    debug_assert!(encoded_metdata_size == HEADER_SIZE);
    debug_assert!(encoded_record_size == meta_data.key_len + meta_data.val_len);

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::core::{
        record::imdb_record::{HEADER_SIZE, ImdbRecord, ImdbRecordMetaData},
        storage::imdb_inline_metadata_format::{decode_record, encode_record},
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
            let meta_data = ImdbRecordMetaData {
                check_sum: 0,
                key_len,
                val_len,
            };
            buffers[i].resize((HEADER_SIZE + key_len + val_len) as usize, b'0');
            let buffer = buffers[i].as_mut_slice();
            encoded_records.push(encode_record(&record, &meta_data, buffer));
            records.push((meta_data, record));
        }

        for i in 0..100 {
            let (decoded_metadata, decoded_record, _) = decode_record(&buffers[i]).unwrap();
            let actual_meta_data = &records[i].0;
            assert_eq!(decoded_metadata.key_len, actual_meta_data.key_len);
            assert_eq!(decoded_metadata.val_len, actual_meta_data.val_len);

            let actual_record = &records[i].1;
            assert_eq!(decoded_record.key, actual_record.key);
            assert_eq!(decoded_record.value, actual_record.value);
        }
    }
}
