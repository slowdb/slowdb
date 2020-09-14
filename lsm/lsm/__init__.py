from typing import Optional
from .memtable import MemTable, Set, Del

class LSM(object):
    def __init__(self, meta_file) -> None:
        self.meta_file = meta_file
        self.memtable = MemTable()

    def lookup(self, k: str) -> Optional[str]:
        return self.memtable.lookup(k)

    def insert(self, k: str, v: str):
        self.memtable.operate(k, Set(v))

    def delete(self, k: str):
        self.memtable.operate(k, Del())
