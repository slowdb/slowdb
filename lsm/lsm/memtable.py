import collections
from typing import Callable, Iterable, Iterator, List, Optional, Sequence, Sized, Tuple, TypeVar


class Op(object): ...


class Set(Op):
    __slots__ = ["value"]
    value: str

    def __init__(self, value: str) -> None:
        self.value = value

    def __eq__(self, o: object) -> bool:
        return isinstance(o, Set) and self.value == o.value


class Del(Op):
    def __eq__(self, o: object) -> bool:
        return isinstance(o, Del)


T = TypeVar("T")
T1 = TypeVar("T1")


# def binary_search(data: List[T], key: T, cmp: Callable[[T, T], int]) -> int:
#     def _go(left: int, right: int):
#         mid = (left + right) // 2
#     return -1


def linear_search(data: Sequence[T1], key: T, cmp: Callable[[T, T1], int]) -> int:
    for i in range(len(data)):
        if cmp(key, data[i]) <= 0:
            return i
    return len(data)


class MemTable(Iterable[Tuple[str, Op]]):
    def __init__(self) -> None:
        self.data = collections.deque()
        self._size = 0

    def _search_key(self, k: str):
        return linear_search(self.data, k, lambda k, k0: -1 if k < k0[0] else 0 if k == k0[0] else 1)

    def lookup(self, k: str) -> Optional[str]:
        i = self._search_key(k)
        if self.data[i][0] == k and isinstance(self.data[i][1], Set):
            return self.data[i][1]

    def operate(self, k: str, op: Op):
        i = self._search_key(k)
        if i != len(self.data) and self.data[i][0] == k:
            self.data[i] = (k, op)
        else:
            self.data.insert(i, (k, op))
            self._size += 1

    def size(self) -> int:
        return self._size

    def __iter__(self) -> Iterator[Tuple[str, Op]]:
        return iter(self.data)


# class MemIter(Iterator[Tuple[str, Op]]):
#     def __init__(self, memTable: MemTable) -> None:
#         self.memTable = memTable

#     def __iter__(self) -> Iterator[Tuple[str, Op]]:
#         return self

#     def __next__(self) -> Tuple[str, Op]:
#         raise StopIteration()
