// keep internal ptr , slides over the buffer
// provides subslices views into the original buffer
pub struct Slicer<'a, B> {
    buffer: B,
    index: u32,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a, B> Slicer<'a, B> {
    pub fn new(buffer: B) -> Self {
        Slicer {
            buffer,
            index: 0,
            _marker: std::marker::PhantomData,
        }
    }

    fn next_range(&mut self, slice_size: u32) -> (usize, usize) {
        let offset = self.index;
        let end = offset + slice_size;

        self.index = end;

        (offset as usize, end as usize)
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }

    pub fn current_index(&self) -> u32 {
        self.index
    }
}

impl<'a> Slicer<'a, &'a [u8]> {
    pub fn next_slice(&mut self, slice_size: u32) -> &[u8] {
        let (offset, end) = self.next_range(slice_size);
        &self.buffer[offset..end]
    }
}

impl<'a> Slicer<'a, &'a mut [u8]> {
    pub fn next_slice_mut(&mut self, slice_size: u32) -> &mut [u8] {
        let (offset, end) = self.next_range(slice_size);
        &mut self.buffer[offset..end]
    }
}
