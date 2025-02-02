from typing import TypeVar, Generic, List, Set, Optional

T = TypeVar('T')

class RecentItems(Generic[T]):
    def __init__(self, max_size: int = 5):
        self._items: List[T] = []
        self._max_size = max_size

    def add(self, item: T) -> None:
        self._items.insert(0, item)

        if len(self._items) > self._max_size:
            self._items.pop()

    def get_items(self) -> List[T]:
        return self._items.copy()

    def get_most_recent(self) -> Optional[T]:
        return self._items[0] if self._items else None

    def get_nth_most_recent(self, n: int) -> Optional[T]:
        if n < 1 or n > len(self._items):
            return None
        return self._items[n - 1]

    def get_most_recent_not_in(self, excluded_set: Set[T]) -> Optional[T]:
        for item in self._items:
            if item not in excluded_set:
                return item
        return None

    def get_nth_most_recent_not_in(self, n: int, excluded_set: Set[T]) -> Optional[T]:
        if n < 1:
            raise ValueError('n must be a positive integer')

        count = 0
        for item in self._items:
            if item not in excluded_set:
                count += 1
                if count == n:
                    return item

        return None
