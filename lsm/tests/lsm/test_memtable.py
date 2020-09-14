from lsm.memtable import MemTable, Set, Del

def test():
    m = MemTable()
    m.operate("b", Set("b"))
    m.operate("a", Set("a"))
    m.operate("c", Set("c"))
    m.operate("b", Del())

    assert list(iter(m)) == [("a", Set("a")), ("b", Del()), ("c", Set("c"))]