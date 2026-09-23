#[derive(Debug)]
pub struct ImdbConfig {
    db_path: String,
}

impl ImdbConfig {
    pub fn new(db_path: String) -> Result<ImdbConfig, String> {
        //! should we validate the path here ? or maybe leave out side parser
        Ok(ImdbConfig { db_path })
    }
}
