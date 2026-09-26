// keep internal ptr , slides over the buffer
// provides subslices views into the original buffer
pub struct Slicer {
    // buffer: &'a [u8],
    index: u32,
}

impl Slicer {
    pub fn new() -> Slicer {
        Slicer { index: 0 }
    }

    pub fn next_slice<'a>(&mut self, buffer: &'a [u8], slice_size: u32) -> &'a [u8] {
        let (offset, end) = self.next_range(slice_size);
        let slice = &buffer[offset..end];
        slice
    }

    pub fn next_slice_mut<'a>(&mut self, buffer: &'a mut [u8], slice_size: u32) -> &'a mut [u8] {
        let (offset, end) = self.next_range(slice_size);
        let slice = &mut buffer[offset..end];
        slice
    }

    fn next_range(&mut self, slice_size: u32) -> (usize, usize) {
        let offset = self.index as usize;
        let end = offset + slice_size as usize;
        self.index += slice_size;

        return (offset, end);
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }

    pub fn current_index(&self) -> u32 {
        self.index
    }
}
