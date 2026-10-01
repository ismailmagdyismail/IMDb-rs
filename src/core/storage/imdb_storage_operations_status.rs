pub type ImdbStorageOperationResult = Result<(), ImdbStorageError>;

pub trait ImdbOperationStatus {
    fn to_string(&self) -> String;
}

pub enum ImdbStorageError {
    CorruptedChecksum,
    PartialRecord,
}

impl ImdbStorageError {
    fn format_error(&self, error_type_str: &str) -> String {
        format!("[Imdb::Storage::Error]:{}", error_type_str)
    }
}

impl ImdbOperationStatus for ImdbStorageError {
    fn to_string(&self) -> String {
        match self {
            ImdbStorageError::CorruptedChecksum => self.format_error("CorruptedChecksum"),
            ImdbStorageError::PartialRecord => self.format_error("PartialRecord"),
        }
    }
}
