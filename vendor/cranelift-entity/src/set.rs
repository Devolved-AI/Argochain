//! Densely numbered entity references as set keys.

use crate::keys::Keys;
use crate::EntityRef;
use alloc::vec::Vec;
use core::marker::PhantomData;

// How many bits are used to represent a single element in `EntitySet`.
const BITS: usize = core::mem::size_of::<usize>() * 8;

/// A set of `K` for densely indexed entity references.
///
/// The `EntitySet` data structure uses the dense index space to implement a set with a bitvector.
/// Like `SecondaryMap`, an `EntitySet` is used to associate secondary information with entities.
#[derive(Debug, Clone)]
pub struct EntitySet<K>
where
    K: EntityRef,
{
    elems: Vec<usize>,
    len: usize,
    unused: PhantomData<K>,
}

impl<K: EntityRef> Default for EntitySet<K> {
    fn default() -> Self {
        Self {
            elems: Vec::new(),
            len: 0,
            unused: PhantomData,
        }
    }
}

/// Shared `EntitySet` implementation for all value types.
impl<K> EntitySet<K>
where
    K: EntityRef,
{
    /// Create a new empty set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new empty set with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            elems: Vec::with_capacity((capacity + (BITS - 1)) / BITS),
            ..Self::new()
        }
    }

    /// Get the element at `k` if it exists.
    pub fn contains(&self, k: K) -> bool {
        let index = k.index();
        if index < self.len {
            (self.elems[index / BITS] & (1 << (index % BITS))) != 0
        } else {
            false
        }
    }

    /// Is this set completely empty?
    pub fn is_empty(&self) -> bool {
        if self.len != 0 {
            false
        } else {
            self.elems.iter().all(|&e| e == 0)
        }
    }

    /// Returns the cardinality of the set. More precisely, it returns the number of calls to
    /// `insert` with different key values, that have happened since the the set was most recently
    /// `clear`ed or created with `new`.
    pub fn cardinality(&self) -> usize {
        self.elems[..(self.len + (BITS - 1)) / BITS]
            .iter()
            .map(|x| x.count_ones() as usize)
            .sum()
    }

    /// Remove all entries from this set.
    pub fn clear(&mut self) {
        self.len = 0;
        self.elems.clear()
    }

    /// Iterate over all the keys in this set.
    pub fn keys(&self) -> Keys<K> {
        Keys::with_len(self.len)
    }

    /// Resize the set to have `n` entries by adding default entries as needed.
    pub fn resize(&mut self, n: usize) {
        self.elems.resize((n + (BITS - 1)) / BITS, 0);
        self.len = n
    }

    /// Insert the element at `k`.
    pub fn insert(&mut self, k: K) -> bool {
        let index = k.index();
        if index >= self.len {
            self.resize(index + 1)
        }
        let result = !self.contains(k);
        self.elems[index / BITS] |= 1 << (index % BITS);
        result
    }

    /// Removes and returns the entity from the set if it exists.
    pub fn pop(&mut self) -> Option<K> {
        if self.len == 0 {
            return None;
        }

        // Clear the last known entity in the list.
        let last_index = self.len - 1;
        self.elems[last_index / BITS] &= !(1 << (last_index % BITS));

        // Set the length to the next last stored entity or zero if we pop'ed
        // the last entity.
        self.len = self
            .elems
            .iter()
            .enumerate()
            .rev()
            .find(|(_, &elem)| elem != 0)
            // Map `i` from `elem` index to bit level index.
            // `(i + 1) * BITS` = Last bit in `elem`.
            // `last - elem.leading_zeros()` = last set bit in `elem`.
            // `as usize` won't ever truncate as the potential range is `0..=8`.
            .map_or(0, |(i, elem)| {
                ((i + 1) * BITS) - elem.leading_zeros() as usize
            });

        Some(K::new(last_index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::u32;

    // `EntityRef` impl for testing.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct E(u32);

    impl EntityRef for E {
        fn new(i: usize) -> Self {
            E(i as u32)
        }
        fn index(self) -> usize {
            self.0 as usize
        }
    }

    #[test]
    fn basic() {
        let r0 = E(0);
        let r1 = E(1);
        let r2 = E(2);
        let mut m = EntitySet::new();

        let v: Vec<E> = m.keys().collect();
        assert_eq!(v, []);
        assert!(m.is_empty());

        m.insert(r2);
        m.insert(r1);

        assert!(!m.contains(r0));
        assert!(m.contains(r1));
        assert!(m.contains(r2));
        assert!(!m.contains(E(3)));
        assert!(!m.is_empty());

        let v: Vec<E> = m.keys().collect();
        assert_eq!(v, [r0, r1, r2]);

        m.resize(20);
        assert!(!m.contains(E(3)));
        assert!(!m.contains(E(4)));
        assert!(!m.contains(E(8)));
        assert!(!m.contains(E(15)));
        assert!(!m.contains(E(19)));

        m.insert(E(8));
        m.insert(E(15));
        assert!(!m.contains(E(3)));
        assert!(!m.contains(E(4)));
        assert!(m.contains(E(8)));
        assert!(!m.contains(E(9)));
        assert!(!m.contains(E(14)));
        assert!(m.contains(E(15)));
        assert!(!m.contains(E(16)));
        assert!(!m.contains(E(19)));
        assert!(!m.contains(E(20)));
        assert!(!m.contains(E(u32::MAX)));

        m.clear();
        assert!(m.is_empty());
    }

    #[test]
    fn pop_ordered() {
        let r0 = E(0);
        let r1 = E(1);
        let r2 = E(2);
        let mut m = EntitySet::new();
        m.insert(r0);
        m.insert(r1);
        m.insert(r2);

        assert_eq!(r2, m.pop().unwrap());
        assert_eq!(r1, m.pop().unwrap());
        assert_eq!(r0, m.pop().unwrap());
        assert!(m.pop().is_none());
        assert!(m.pop().is_none());
    }

    #[test]
    fn pop_unordered() {
        let mut blocks = [
            E(0),
            E(1),
            E(6),
            E(7),
            E(5),
            E(9),
            E(10),
            E(2),
            E(3),
            E(11),
            E(12),
        ];

        let mut m = EntitySet::new();
        for &block in &blocks {
            m.insert(block);
        }
        assert_eq!(m.len, 13);
        blocks.sort();

        for &block in blocks.iter().rev() {
            assert_eq!(block, m.pop().unwrap());
        }

        assert!(m.is_empty());
    }

    #[test]
    fn cardinality() {
        let mut m = EntitySet::new();

        m.insert(E(1));
        assert!(m.cardinality() == 1);

        m.insert(E(0));
        assert!(m.cardinality() == 2);

        m.insert(E(1));
        assert!(m.cardinality() == 2);

        m.insert(E(BITS as u32 - 1));
        assert!(m.cardinality() == 3);

        m.insert(E(BITS as u32));
        assert!(m.cardinality() == 4);

        m.insert(E(BITS as u32 - 1));
        assert!(m.cardinality() == 4);

        assert!(m.pop() == Some(E(BITS as u32)));
        assert!(m.cardinality() == 3);
        assert!(m.pop() == Some(E(BITS as u32 - 1)));
        assert!(m.cardinality() == 2);

        m.insert(E(100));
        assert!(m.cardinality() == 3);

        assert!(m.pop() == Some(E(100)));
        assert!(m.cardinality() == 2);

        m.insert(E(100));
        m.insert(E(101));
        m.insert(E(102));
        assert!(m.cardinality() == 5);
    }
}
