pub type ImdbStorageOperationResult = Result<(), ImdbStorageError>;

pub trait ImdbOperationStatus {
    fn to_string(&self) -> String;
}

#[derive(Debug)]
pub enum ImdbStorageError {
    CorruptedChecksum,
    PartialRecordRead(u32, u32),
    DecodeInSufficientBufferSize(&'static str),
    EncodeInSufficientBufferSize(&'static str),
    StorageEntryCorruption(&'static str),
    BufferFlush(String),
    Fsync(String),
    DiskWrite(String),
    DiskSeek(&'static str, String),
    FileOpen(String),
    DiskRead(&'static str, String),
    StorageDirectory(String),
    InconsistentDiskWithIndex,
}

impl ImdbStorageError {
    fn format_error(&self, error_type_str: &str, extra_info: &str) -> String {
        format!("[Imdb::Storage::Error::{}]: {}", error_type_str, extra_info)
    }
}

impl ImdbOperationStatus for ImdbStorageError {
    fn to_string(&self) -> String {
        match self {
            ImdbStorageError::CorruptedChecksum => self.format_error("CorruptedChecksum", ""),
            ImdbStorageError::PartialRecordRead(expected_size, actual_size) => self.format_error(
                "PartialRecordRead",
                &format!(
                    "expected {} bytes found {} bytes",
                    expected_size, actual_size
                ),
            ),
            ImdbStorageError::DecodeInSufficientBufferSize(buffer_name) => {
                self.format_error("DecodeInSufficientBufferSize", buffer_name)
            }
            ImdbStorageError::EncodeInSufficientBufferSize(buffer_name) => {
                self.format_error("EncodeInSufficientBufferSize", buffer_name)
            }
            ImdbStorageError::StorageEntryCorruption(entry_name) => {
                self.format_error("StorageEntryCorruption", entry_name)
            }
            ImdbStorageError::BufferFlush(error_details) => {
                self.format_error("BufferFlush", error_details)
            }
            ImdbStorageError::Fsync(error_details) => self.format_error("Fsync", error_details),
            ImdbStorageError::DiskWrite(error_details) => {
                self.format_error("DiskWrite", error_details)
            }
            ImdbStorageError::DiskSeek(offset_required, error_details) => self.format_error(
                "DiskWrite",
                &format!(
                    "Error happend while seeking offset {}, {}",
                    offset_required, error_details,
                ),
            ),
            ImdbStorageError::FileOpen(error_details) => {
                self.format_error("FileOpen", error_details)
            }
            ImdbStorageError::DiskRead(read_entry_name, error_details) => self.format_error(
                "DiskRead",
                &format!(
                    "Error happend while reading {}, {}",
                    read_entry_name, error_details,
                ),
            ),
            ImdbStorageError::StorageDirectory(error_details) => {
                self.format_error("StorageDirectory", error_details)
            }
            ImdbStorageError::InconsistentDiskWithIndex => {
                self.format_error("InconsistentDiskWithIndex", "")
            }
        }
    }
}
