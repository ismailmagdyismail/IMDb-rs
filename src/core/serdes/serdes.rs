use crate::core::serdes::slicer::Slicer;

pub struct Serializer<'a> {
    slicer: Slicer<'a, &'a mut [u8]>,
}

impl<'a> Serializer<'a> {
    pub fn new(sink_buffer: &'a mut [u8]) -> Self {
        Serializer {
            slicer: Slicer::new(sink_buffer),
        }
    }

    pub fn serialize(&mut self, element: &[u8]) -> &mut Self {
        let dest_slice = self.slicer.next_slice_mut(element.len() as u32);
        dest_slice.copy_from_slice(element);
        self
    }

    pub fn size(&self) -> u32 {
        return self.slicer.current_index();
    }
}

pub struct Deserilizer<'a> {
    slicer: Slicer<'a, &'a [u8]>,
}

impl<'a> Deserilizer<'a> {
    pub fn new(src_buffer: &'a [u8]) -> Deserilizer<'a> {
        Deserilizer {
            slicer: Slicer::new(src_buffer),
        }
    }

    pub fn deserialize_copy(&mut self, size: u32) -> Vec<u8> {
        let src_slice = self.slicer.next_slice(size);
        let sink_slice = Vec::from(src_slice);
        sink_slice
    }

    pub fn size(&self) -> u32 {
        return self.slicer.current_index();
    }
}
