use crate::core::serdes::slicer::Slicer;

pub struct Serializer<'a> {
    sink_buffer: &'a mut [u8],
    slicer: Slicer,
}

impl<'a> Serializer<'a> {
    pub fn new(sink_buffer: &'a mut [u8]) -> Serializer<'a> {
        Serializer {
            sink_buffer,
            slicer: Slicer::new(),
        }
    }

    pub fn serialize(&mut self, element: &[u8]) -> &mut Serializer<'a> {
        let dest_slice = self
            .slicer
            .next_slice_mut(self.sink_buffer, element.len() as u32);
        dest_slice.copy_from_slice(element);
        self
    }

    pub fn size(&self) -> u32 {
        return self.slicer.current_index();
    }
}

pub struct Deserilizer<'a> {
    src_buffer: &'a [u8],
    slicer: Slicer,
}

impl<'a> Deserilizer<'a> {
    pub fn new(src_buffer: &'a [u8]) -> Deserilizer<'a> {
        Deserilizer {
            src_buffer,
            slicer: Slicer::new(),
        }
    }

    pub fn deserialize_copy(&mut self, size: u32) -> Vec<u8> {
        let src_slice = self.slicer.next_slice(&self.src_buffer, size);
        let sink_slice = Vec::from(src_slice);
        sink_slice
    }

    pub fn size(&self) -> u32 {
        return self.slicer.current_index();
    }
}
