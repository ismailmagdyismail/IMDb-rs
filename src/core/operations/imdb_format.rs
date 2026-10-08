pub enum ImdbFormat {
    InlineMetadata,
    SeperateMetadata,
}

impl ImdbFormat {
    pub fn from_str(format: &str) -> Option<Self> {
        match format {
            "inline_metadata" => Some(ImdbFormat::InlineMetadata),
            "seperate_metadata" => Some(ImdbFormat::SeperateMetadata),
            _ => None,
        }
    }
}
