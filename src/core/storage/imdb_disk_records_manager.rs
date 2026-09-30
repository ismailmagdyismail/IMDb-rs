use crate::core::{
    record::imdb_record::ImdbRecord,
    storage::{
        pager::{ImdbRecordMetadataStorageEntry, Offset},
        writer::ImdbStorageEntry,
    },
};

// I am not sure yet about the abtraction level here
// should Index be:
//      A. a part of the disk layer ?
//         - conflicts / add multiple responsisbilites to the disk layer
//         - won't be easy to add Secondary indexes later
//         - could be done as if disk layer shipped with one primary index ,not tunable
//         - or we could simplfy leave it up to the user of disk layer to add his own indexes
//         - also adding it to disk layer couples execution to 1 path
//           query is always executed using that 1 flow / path
//           Query planner if added cannot decide a more effcient way to do it
//           always goes to disk, if we were to add BufferPool / PageCache
//           we would have to insert it within disk layer
//
//      B. a higher layer that makes use of disk layer
//          - this model yet simple, COUPLES index, disk layer
//          - execution of query is done in 1 way now, cannot be optimized
//          - IFF we werer to introduce BufferPool/PageCache to cache records on disk
//            instead of going to the disk on each read, then index code would have to change
//            to fetch from buffer pool, instead of on Disk pager
//            this COUPLES them together and puts many execution path into index
//            causing COMPLEXITY, and not easy to ADD NEW exeuction FLOWS
//          - any new layers added would have to be inserted between them
//
//      C. a Sibiling layer, which is coordinated with disk layer via another
//      3rd layer that sits on top of them both
//         - we provide a set of gurantees for layers above
//            [Disk Access for Record Data files]
//         - other functionalities as (Indexes, WAL, Checkpointing, Transactions)
//            are implemented in terms of interface provided
//

pub trait ImdbDiskRecordsManager {
    fn write_record(&mut self, record: &ImdbRecord) -> Result<ImdbStorageEntry, String>;

    fn sync(&mut self) -> Result<(), String>;

    fn read_record_with_id_offset(
        &mut self,
        offset: Offset,
    ) -> Result<Option<ImdbRecordMetadataStorageEntry>, String>;

    fn read_next_record(&mut self) -> Result<Option<ImdbRecordMetadataStorageEntry>, String>;

    // a More generic, better API I think
    // returns a cursor / Iterator like
    // thread safe, since read only cursor
    // can iterate over records as needed by caller
    // fn create_scan_cursor()
}
