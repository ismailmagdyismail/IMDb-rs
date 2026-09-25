pub struct Serializer<'a> {
    sink_buffer: &'a mut [u8],
    index: u32,
}

impl<'a> Serializer<'a> {
    pub fn new(sink_buffer: &'a mut [u8]) -> Serializer<'a> {
        Serializer {
            sink_buffer,
            index: 0,
        }
    }

    pub fn serialize(&mut self, element: &[u8]) -> &mut Serializer<'a> {
        let offset = self.index;
        let size = element.len() as u32;
        let end = offset + size;

        let dest_slice = &mut self.sink_buffer[offset as usize..end as usize];
        dest_slice.copy_from_slice(element);

        self.index += size;

        self
    }

    pub fn size(&self) -> u32 {
        return self.index;
    }
}

pub struct Deserilizer<'a> {
    src_buffer: &'a [u8],
    index: u32,
}

impl<'a> Deserilizer<'a> {
    pub fn new(src_buffer: &'a [u8]) -> Deserilizer<'a> {
        Deserilizer {
            src_buffer,
            index: 0,
        }
    }

    pub fn deserialize_copy(&mut self, size: u32) -> Vec<u8> {
        let offset = self.index;
        let end = offset + size;
        let src_slice = &self.src_buffer[offset as usize..end as usize];
        let sink_slice = Vec::from(src_slice);
        self.index += size;
        sink_slice
    }

    pub fn size(&self) -> u32 {
        return self.index;
    }
}
