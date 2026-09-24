use crate::core::record::imdb_record::ImdbRecord;

impl ImdbRecord {
    pub fn ser_size(&self) -> u32 {
        return self.key.len() as u32 + self.value.len() as u32;
    }

    pub fn serialize(&self, buffer: &mut Vec<u8>) -> Result<u32, String> {
        let required_size = self.ser_size();
        if (buffer.len() as u32) < required_size {
            return Result::Err("[Buffer for serdes is too small]".to_string());
        }

        let key_len: usize = self.key.len();
        let key_offset = 0;
        let key_end: usize = key_offset + key_len;
        let dest_key_buffer_bytes = &mut buffer[key_offset..key_end];
        dest_key_buffer_bytes.copy_from_slice(self.key.as_bytes());

        let value_offset = key_end;
        let value_len = self.value.len();
        let value_end = value_offset + value_len;
        let dest_value_buffer_bytes = &mut buffer[value_offset..value_end];
        dest_value_buffer_bytes.copy_from_slice(self.value.as_bytes());

        Result::Ok(key_len as u32 + value_len as u32)
    }
}

impl ImdbRecord {
    // pub fn deserialize_copy(buffer: &Vec<u8>) -> Result<(ImdbRecord, u32), String> {
    // }

    // for zero copy deserialization
    // pub fn deseialize_owned()
}

#[cfg(test)]
mod test {
    use crate::core::record::imdb_record::ImdbRecord;

    #[test]
    fn test_size_required() {
        let record = ImdbRecord {
            key: "1".to_string(),
            value: "ismail".to_string(),
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
            key: "1".to_string(),
            value: "ismail".to_string(),
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
}
