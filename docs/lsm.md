# LSM Tree Implementation

LSM (Log Structured Merge tree) is a **write efficient** / **immutable** structure for keyed data storage.

A simple LMS tree comprises of two B+ Tree with one in memory and another on disk.

## Implementation (Phase 1)

* memtable
* immutable memtable
* lookup
* sstable
* merge

## Implementation (Phase 2)

* compaction
* wal
* pending improvements