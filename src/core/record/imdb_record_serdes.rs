use crate::core::{
    record::imdb_record::{ImdbRecord, ImdbRecordMetaData},
    serdes::serdes::{Deserilizer, Serializer},
};

impl ImdbRecord {
    pub fn ser_size(&self) -> u32 {
        return self.key.len() as u32 + self.value.len() as u32;
    }

    pub fn serialize(&self, sink_buffer: &mut Vec<u8>) -> Result<u32, String> {
        let required_size = self.ser_size();
        if (sink_buffer.len() as u32) < required_size {
            return Result::Err("[Buffer for serialization is too small]".to_string());
        }

        let mut serializer = Serializer::new(sink_buffer);
        serializer.serialize(&self.key).serialize(&self.value);

        debug_assert!(self.ser_size() == serializer.size());

        Result::Ok(serializer.size())
    }
}

impl ImdbRecord {
    pub fn deserialize_copy(
        meta_data: &ImdbRecordMetaData,
        src_buffer: &Vec<u8>,
    ) -> Result<(ImdbRecord, u32), String> {
        if meta_data.key_len + meta_data.val_len > src_buffer.len() as u32 {
            return Result::Err("[Buffer for derserilization is too small]".to_string());
        }
        let mut deserializer = Deserilizer::new(&src_buffer);
        let key_sink_buffer = deserializer.deserialize_copy(meta_data.key_len);
        let value_sink_buffer = deserializer.deserialize_copy(meta_data.val_len);

        let record = ImdbRecord {
            key: key_sink_buffer,
            value: value_sink_buffer,
        };

        Ok((record, deserializer.size()))
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
