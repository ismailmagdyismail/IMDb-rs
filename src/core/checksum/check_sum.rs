pub trait CheckSum {
    fn calculate(&self, bytes: &[u8]) -> u32;
}
