# TODO(s)

## Tools

- [DONE] port over hex viewer , make it specific to this file format, to help with debugging
- [] tool to manipulate Records and Index files (set record x with y), could be used in fault, error injection
- [] an aggregate tool composing all sub tools through one exe

## Serdes

- [DONE] create abstraction for serdes
- [DONE] change kv to be [u8] / vec[u8]

## Operations

- [] help command
- [] load a file and run all commands in it

## Testing

- [] DST

## Persistance

### Physical records

- [DONE] create in memory index
- [DONE] update in memory index
- [DONE] support records spanning multiple buffer reads
- [DONE] (flush & fsync) write record to disk
- [DONE] Reader should panic or error in case an entry in cache is not found on disk by pager
- [DONE] Storage Engine tests Integration tests (ImdbEngine [StorageEngine + Index])
- [] Deleting records in index
- [] test loading db after deleting record x (it will be re-added in index) cause last entry override index entry , so must add tombstone marker (Deleting records from physical page / tombestone )
<!-- - [] inline meta-data vs seperate meta data and payload -->

### Corruption

- [DONE] calculate CRC
- [DONE] Detect corrupted entry
- [] Log truncation on Recovery
- [] Non happy Storage Reader, Writer tests (reading at offset in middle of a record | incomplete record)

### Index Durability

### Transactions

- NOTE: depedant on WAL | COW
- [] transactions api
- [] atomicity using WAL
- [] test isolation
- [] transactions batch file

### WAL and Async flush and Concurrency

#### WAL

- [] write data to WAL Synchronously
- [] BG thread for asynchrnous writes to index, data files
- [] time travel

#### MVCC

- [] MVCC, How does multiple physical records fit within the DB file (are they written in same file)
- [] primary indecies may require updating to handle which version is visible
- [] Define concepts of pages if needed Coordination with Page Cache

#### Steal, Force, Flush policies

- [] STEAL, Force Polices
- [] Shadow Paging, Swapping

### Checkpointing

-

## Research

- [] is index coupled to physical layout on disk, if no, then what are pros of cons of coupling vs decoupling them
- [] corrupted record => truncation ? | deleting just 1 corrupted record / tombstoning it
