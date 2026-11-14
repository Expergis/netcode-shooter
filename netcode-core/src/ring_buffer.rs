//! Fixed-size circular ring buffer for storing historical states without heap allocations.

use std::mem::MaybeUninit;

/// A fixed-capacity, zero-heap-allocation circular buffer.
///
/// `N` must be a power-of-two to enable fast bitmask indexing.
pub struct RingBuffer<T, const N: usize> {
    storage: [MaybeUninit<T>; N],
    head: usize,
    len: usize,
}

impl<T, const N: usize> RingBuffer<T, N> {
    /// Constructs an empty [`RingBuffer`].
    pub fn new() -> Self {
        debug_assert!(N > 0 && N.is_power_of_two(), "N must be a power of two!");
        let storage = unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() };
        Self {
            storage,
            head: 0,
            len: 0,
        }
    }

    /// Returns the capacity `N`.
    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Returns the number of valid items currently stored.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the buffer contains no valid items.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns `true` if the buffer is at capacity `N`.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len == N
    }

    #[inline]
    /// Returns the 'physical' index from the 'logical' index
    fn physical(&self, i: usize) -> usize {
        (self.head + N - self.len + i) & (N - 1)
    }

    /// Appends `value` to the buffer, overwriting the oldest entry if full.
    pub fn push(&mut self, value: T) {
        let slot = &mut self.storage[self.head];
        if self.len == N {
            // SAFETY: when full, every slot is initialized, including `head` (which is the oldest when len == N).
            unsafe { slot.assume_init_drop() };
        } else {
            self.len += 1;
        }
        slot.write(value);
        self.head = (self.head + 1) & (N - 1);
    }

    /// Access item at logical index `i` (0 = oldest, `len - 1` = newest).
    pub fn get(&self, i: usize) -> Option<&T> {
        if i < self.len {
            Some(unsafe { self.storage[self.physical(i)].assume_init_ref() })
        } else {
            None
        }
    }

    /// Returns reference to the newest entry.
    pub fn newest(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            self.get(self.len - 1)
        }
    }

    /// Returns reference to the oldest entry.
    pub fn oldest(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            self.get(0)
        }
    }

    /// Returns an iterator over all valid elements from oldest to newest.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        (0..self.len).map(move |i| {
            // SAFETY: i < len.
            unsafe { self.storage[self.physical(i)].assume_init_ref() }
        })
    }
}

impl<T, const N: usize> Default for RingBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<T: Send, const N: usize> Send for RingBuffer<T, N> {}
unsafe impl<T: Sync, const N: usize> Sync for RingBuffer<T, N> {}

impl<T, const N: usize> Drop for RingBuffer<T, N> {
    fn drop(&mut self) {
        for i in 0..self.len {
            let idx = self.physical(i);
            // SAFETY: each logical index < len maps to an initialized slot, dropped exactly once.
            unsafe { self.storage[idx].assume_init_drop() };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_basic() {
        let mut buf: RingBuffer<u32, 4> = RingBuffer::new();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);

        buf.push(10);
        buf.push(20);
        buf.push(30);

        assert_eq!(buf.len(), 3);
        assert_eq!(buf.oldest(), Some(&10));
        assert_eq!(buf.newest(), Some(&30));
        assert_eq!(buf.get(0), Some(&10));
        assert_eq!(buf.get(1), Some(&20));
        assert_eq!(buf.get(2), Some(&30));
        assert_eq!(buf.get(3), None);
    }

    #[test]
    fn test_ring_buffer_wrap_around() {
        let mut buf: RingBuffer<u32, 4> = RingBuffer::new();
        buf.push(1);
        buf.push(2);
        buf.push(3);
        buf.push(4);
        assert!(buf.is_full());

        // Overwrite 1 with 5
        buf.push(5);
        assert_eq!(buf.len(), 4);
        assert_eq!(buf.oldest(), Some(&2));
        assert_eq!(buf.newest(), Some(&5));

        let collected: Vec<u32> = buf.iter().copied().collect();
        assert_eq!(collected, vec![2, 3, 4, 5]);
    }
}
