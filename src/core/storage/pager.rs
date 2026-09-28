use crate::core::record::imdb_record::{ImdbRecord, ImdbRecordMetaData};

pub type Offset = u64;

#[derive(Debug)]
pub struct ImdbRecordMetadataStorageEntry {
    pub metadata: ImdbRecordMetaData,
    pub record: ImdbRecord,

    // a bit of a leaky abstraction (exposing physical layout format) to return both to the caller
    pub record_offset: Offset,
    pub metadata_offset: Offset,

    // this comes back to the idea of [Handles vs Ptrs] and [Physical vs Logical] and like [Page ID , Slot-ID]
    // we will stick to indetfying offset instead of a generic id JUST for simplicity
    // crucial offset decouples index layer from physical storage layer
    // depending on underlying storage format it means differnt things
    // interperation of this offset is lift up to the storage layer
    // the Contract of this offset mean
    //      - to id a certain record, use this offset
    //      - could use it for caching
    //      - if you want to re-access that record, you supply that offset
    //      - that offset would be used to fetch physical record
    //      - it may be fetched using 1 or more disk seeks depending on underlying storage
    // examples
    //      - for Inline-Metadata   -> it means meta-data offset which then fetches next key_len + value_len bytes,
    //      - for Seperate-Metadata -> it means record offset directly, to avoid 2 Disk seeks
    // it may be generalized to always use / return meta-data offset,  but this would result in worse performance
    // on some storage layouts may result in mutliple disk look ups
    // for example on  "Seperate-Metadata" engine it will result in 2 disk seeks (1 for metadata , 1 for physical records)
    // we return full info to caller anyway, cause we may add different APIs later that are more optimal if supplied more info
    pub identfying_offset: Offset,
}

pub trait ImdbRecordPager {
    fn load_next_record_and_metadata(
        &mut self,
    ) -> Result<Option<ImdbRecordMetadataStorageEntry>, String>;

    fn load_specific_record_and_meta_data_using_id_offset(
        &mut self,
        offset: Offset,
    ) -> Result<Option<ImdbRecordMetadataStorageEntry>, String>;
}
