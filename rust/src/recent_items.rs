use std::collections::HashSet;
use std::hash::Hash;

/// A bounded list that tracks recently added items.
/// Items are stored most-recent-first.
#[derive(Debug, Clone)]
pub struct RecentItems<T> {
    items: Vec<T>,
    max_size: usize,
}

impl<T> Default for RecentItems<T> {
    fn default() -> Self {
        Self::new(5)
    }
}

impl<T> RecentItems<T> {
    /// Create a new RecentItems with the specified maximum size.
    pub fn new(max_size: usize) -> Self {
        Self {
            items: Vec::with_capacity(max_size),
            max_size,
        }
    }

    /// Add an item to the front of the list.
    /// If the list exceeds max_size, the oldest item is removed.
    pub fn add(&mut self, item: T) {
        self.items.insert(0, item);
        if self.items.len() > self.max_size {
            self.items.pop();
        }
    }

    /// Get a copy of all items.
    pub fn get_items(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.items.clone()
    }

    /// Get the most recently added item.
    pub fn get_most_recent(&self) -> Option<&T> {
        self.items.first()
    }

    /// Get the nth most recently added item (1-indexed).
    pub fn get_nth_most_recent(&self, n: usize) -> Option<&T> {
        if n < 1 || n > self.items.len() {
            return None;
        }
        Some(&self.items[n - 1])
    }

    /// Get the most recently added item that is not in the excluded set.
    pub fn get_most_recent_not_in(&self, excluded_set: &HashSet<T>) -> Option<&T>
    where
        T: Eq + Hash,
    {
        for item in &self.items {
            if !excluded_set.contains(item) {
                return Some(item);
            }
        }
        None
    }

    /// Get the nth most recently added item that is not in the excluded set (1-indexed).
    pub fn get_nth_most_recent_not_in(&self, n: usize, excluded_set: &HashSet<T>) -> Option<&T>
    where
        T: Eq + Hash,
    {
        if n < 1 {
            return None;
        }

        let mut count = 0;
        for item in &self.items {
            if !excluded_set.contains(item) {
                count += 1;
                if count == n {
                    return Some(item);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get_most_recent() {
        let mut recent: RecentItems<i32> = RecentItems::new(3);
        recent.add(1);
        recent.add(2);
        recent.add(3);
        assert_eq!(recent.get_most_recent(), Some(&3));
    }

    #[test]
    fn test_max_size() {
        let mut recent: RecentItems<i32> = RecentItems::new(3);
        recent.add(1);
        recent.add(2);
        recent.add(3);
        recent.add(4);
        assert_eq!(recent.get_items(), vec![4, 3, 2]);
    }

    #[test]
    fn test_get_nth_most_recent() {
        let mut recent: RecentItems<i32> = RecentItems::new(5);
        recent.add(1);
        recent.add(2);
        recent.add(3);
        assert_eq!(recent.get_nth_most_recent(1), Some(&3));
        assert_eq!(recent.get_nth_most_recent(2), Some(&2));
        assert_eq!(recent.get_nth_most_recent(3), Some(&1));
        assert_eq!(recent.get_nth_most_recent(4), None);
        assert_eq!(recent.get_nth_most_recent(0), None);
    }

    #[test]
    fn test_get_nth_most_recent_not_in() {
        let mut recent: RecentItems<i32> = RecentItems::new(5);
        recent.add(1);
        recent.add(2);
        recent.add(3);
        recent.add(4);

        let mut excluded = HashSet::new();
        excluded.insert(4);
        excluded.insert(2);

        assert_eq!(recent.get_nth_most_recent_not_in(1, &excluded), Some(&3));
        assert_eq!(recent.get_nth_most_recent_not_in(2, &excluded), Some(&1));
        assert_eq!(recent.get_nth_most_recent_not_in(3, &excluded), None);
    }
}
