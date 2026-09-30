use crate::core::{
    record::imdb_record::{
        ImdbRecord, ImdbRecordMetaData, KEY_LEN_SIZE, META_DATA_SIZE, VAL_LEN_SIZE,
    },
    serdes::serdes::{Deserilizer, Serializer},
};

/*
- this is somewhat coupled to the underlying storage engine format
- we cannot easily change the format to use columnar oriented format for example
- THUS it should be moved out of here to the storage_engine/imdb_inline_format.rs
- since each storage_engine could decide its own format
- EX_1: inline_meta data => [checksum, header, payload]
- EX_2: columnar => [header_0,header_1,header_3] , [payload_0,payload_1,payload_2]
the current serdes makes an IMPLICIT choice about the underlying engine
*/

impl ImdbRecord {
    pub fn ser_size(&self) -> u32 {
        return self.key.len() as u32 + self.value.len() as u32;
    }

    pub fn serialize(&self, sink_buffer: &mut [u8]) -> Result<u32, String> {
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
        src_buffer: &[u8],
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

impl ImdbRecordMetaData {
    pub fn ser_size(&self) -> u32 {
        return KEY_LEN_SIZE + VAL_LEN_SIZE;
    }

    pub fn serialize(&self, sink_buffer: &mut [u8]) -> Result<u32, String> {
        let required_size = self.ser_size();
        if (sink_buffer.len() as u32) < required_size {
            return Result::Err("[Buffer for Meta-Data serialization is too small]".to_string());
        }

        let mut serializer = Serializer::new(sink_buffer);
        serializer
            .serialize(&self.key_len.to_le_bytes())
            .serialize(&self.val_len.to_le_bytes());

        debug_assert!(self.ser_size() == serializer.size());

        Result::Ok(serializer.size())
    }

    pub fn deserialize_copy(src_buffer: &[u8]) -> Result<(ImdbRecordMetaData, u32), String> {
        if META_DATA_SIZE > src_buffer.len() as u32 {
            return Result::Err("[Buffer for Meta-Data derserilization is too small]".to_string());
        }

        let mut deserializer = Deserilizer::new(&src_buffer);
        let key_sink_buffer = deserializer.deserialize_copy(KEY_LEN_SIZE as u32);
        let value_sink_buffer = deserializer.deserialize_copy(VAL_LEN_SIZE as u32);

        let key_len = u32::from_le_bytes(
            key_sink_buffer
                .try_into()
                .map_err(|_| "Corrupted key_len Entry")?,
        );
        let val_len = u32::from_le_bytes(
            value_sink_buffer
                .try_into()
                .map_err(|_| "Corrupted value_len Entry")?,
        );

        let metadata = ImdbRecordMetaData { key_len, val_len };
        Ok((metadata, deserializer.size()))
    }
}

#[cfg(test)]
mod test {
    use crate::core::record::imdb_record::{ImdbRecord, ImdbRecordMetaData, META_DATA_SIZE};

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
            key_len: key_len as u32,
            val_len: value_len as u32,
        };
        let (des_record, des_size) = ImdbRecord::deserialize_copy(&meta_data, &ser_buffer).unwrap();

        assert_eq!(des_size, ser_required_size);
        assert_eq!(record.key, des_record.key);
        assert_eq!(record.value, des_record.value);
    }

    #[test]
    fn test_metadata_serialization() {
        let key_len = 1;
        let val_len = 6;
        let meta_data = ImdbRecordMetaData { key_len, val_len };
        let expected_ser_size = meta_data.ser_size();
        assert_eq!(expected_ser_size, META_DATA_SIZE);

        let mut buffer = [b'0'; META_DATA_SIZE as usize];
        let actual_ser_size = meta_data.serialize(&mut buffer).unwrap();
        assert_eq!(expected_ser_size, actual_ser_size);
    }

    #[test]
    fn test_written_metadata_bytes() {
        let key_len = 1;
        let val_len = 6;
        let meta_data = ImdbRecordMetaData { key_len, val_len };
        let expected_ser_size = meta_data.ser_size();
        assert_eq!(expected_ser_size, META_DATA_SIZE);

        let mut buffer = [b'0'; META_DATA_SIZE as usize];
        let actual_ser_size = meta_data.serialize(&mut buffer).unwrap();
        assert_eq!(expected_ser_size, actual_ser_size);

        let expected_bytes = [
            1u32.to_le_bytes(), // key_len
            6u32.to_le_bytes(), // val_len
        ]
        .concat();
        for i in 0..expected_bytes.len() {
            assert_eq!(expected_bytes[i], buffer[i]);
        }
    }

    #[test]
    fn test_metadata_deserialzation() {
        let key_len: u32 = 1;
        let val_len: u32 = 6;
        let ser_buffer = [key_len.to_le_bytes(), val_len.to_le_bytes()].concat();

        let (des_metadata, des_size) = ImdbRecordMetaData::deserialize_copy(&ser_buffer).unwrap();
        assert_eq!(des_size, META_DATA_SIZE);
        assert_eq!(des_metadata.key_len, key_len);
        assert_eq!(des_metadata.val_len, val_len);
    }

    #[test]
    fn test_metadata_serdes() {
        let key_len = 1;
        let val_len = 6;
        let metadata = ImdbRecordMetaData { key_len, val_len };

        let mut ser_buffer = vec![b'0'; META_DATA_SIZE as usize];
        let ser_size = metadata.serialize(&mut &mut ser_buffer).unwrap();
        assert_eq!(ser_size, META_DATA_SIZE);

        let (des_metadata, des_size) = ImdbRecordMetaData::deserialize_copy(&ser_buffer).unwrap();
        assert_eq!(des_size, META_DATA_SIZE);

        assert_eq!(des_metadata.key_len, key_len);
        assert_eq!(des_metadata.val_len, val_len);
    }
}
