use crate::core::record::imdb_record::{ImdbRecord, ImdbRecordMetaData};

impl ImdbRecord {
    pub fn ser_size(&self) -> u32 {
        return self.key.len() as u32 + self.value.len() as u32;
    }

    pub fn serialize(&self, buffer: &mut Vec<u8>) -> Result<u32, String> {
        let required_size = self.ser_size();
        if (buffer.len() as u32) < required_size {
            return Result::Err("[Buffer for serialization is too small]".to_string());
        }

        let key_len: usize = self.key.len();
        let key_offset = 0;
        let key_end: usize = key_offset + key_len;
        let dest_key_buffer_bytes = &mut buffer[key_offset..key_end];
        dest_key_buffer_bytes.copy_from_slice(self.key.as_slice());

        let value_offset = key_end;
        let value_len = self.value.len();
        let value_end = value_offset + value_len;
        let dest_value_buffer_bytes = &mut buffer[value_offset..value_end];
        dest_value_buffer_bytes.copy_from_slice(self.value.as_slice());

        Result::Ok(key_len as u32 + value_len as u32)
    }
}

impl ImdbRecord {
    pub fn deserialize_copy(
        meta_data: &ImdbRecordMetaData,
        buffer: &Vec<u8>,
    ) -> Result<(ImdbRecord, u32), String> {
        if meta_data.key_len + meta_data.val_len > buffer.len() as u32 {
            return Result::Err("[Buffer for derserilization is too small]".to_string());
        }
        let key_offset = 0 as usize;
        let key_size = meta_data.key_len as usize;
        let key_end = key_offset + key_size;
        let src_key_slice = &buffer[key_offset..key_end];
        let deserialized_key = Vec::from(src_key_slice);

        let value_offset = key_end as usize;
        let value_size = meta_data.val_len as usize;
        let value_end = value_offset + value_size as usize;
        let src_value_slice = &buffer[value_offset..value_end];
        let deserialized_value = Vec::from(src_value_slice);

        let record = ImdbRecord {
            key: deserialized_key,
            value: deserialized_value,
        };

        Ok((record, (key_size + value_size) as u32))
    }

    // for zero copy deserialization, should either
    // A. take ownership of the passed buffer
    // B. creata a struct with borrowed members
    // pub fn deseialize_owned()
}

#[cfg(test)]
mod test {
    use crate::core::record::imdb_record::{ImdbRecord, ImdbRecordMetaData};

    #[test]
    fn test_size_required() {
        let record = ImdbRecord {
            key: "1".as_bytes().to_vec(),
            value: "ismail".as_bytes().to_vec(),
        };
        let required_size = record.ser_size();
        assert_eq!(required_size, 7);

        let mut buffer: Vec<u8> = vec![b'0'; required_size as usize];
        let actual_size = record.serialize(&mut buffer).unwrap();
        assert_eq!(required_size, actual_size);
    }

    #[test]
    fn test_written_bytes() {
        let record = ImdbRecord {
            key: "1".as_bytes().to_vec(),
            value: "ismail".as_bytes().to_vec(),
        };
        let required_size = record.ser_size();
        assert_eq!(required_size, 7);

        let mut buffer: Vec<u8> = vec![b'0'; (required_size * 10) as usize];
        let actual_size = record.serialize(&mut buffer).unwrap();
        assert_eq!(required_size, actual_size);

        let expected_bytes = "1ismail".as_bytes();
        for i in 0..required_size as usize {
            assert_eq!(buffer[i], expected_bytes[i]);
        }
    }

    #[test]
    fn test_deserialization() {
        let key = "1".as_bytes().to_vec();
        let value = "ismail".as_bytes().to_vec();
        let key_len = key.len();
        let value_len = value.len();
        let record = ImdbRecord { key, value };
        let ser_required_size = record.ser_size();
        assert_eq!(ser_required_size, 7);

        let mut ser_buffer = vec![b'0'; (ser_required_size) as usize];
        let actual_size = record.serialize(&mut ser_buffer).unwrap();
        assert_eq!(ser_required_size, actual_size);

        let meta_data = ImdbRecordMetaData {
            check_sum: 0,
            key_len: key_len as u32,
            val_len: value_len as u32,
        };
        let (des_record, des_size) = ImdbRecord::deserialize_copy(&meta_data, &ser_buffer).unwrap();

        assert_eq!(des_size, ser_required_size);
        assert_eq!(record.key, des_record.key);
        assert_eq!(record.value, des_record.value);
    }
}
