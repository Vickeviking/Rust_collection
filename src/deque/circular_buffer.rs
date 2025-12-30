use std::{marker::PhantomData, mem::MaybeUninit};

struct CircularBuffer<T> {
    data: Vec<MaybeUninit<T>>,
    start: usize,
    len: usize,
}

pub struct Iter<'a, T> {
    buf: &'a CircularBuffer<T>,
    idx: usize, //logical index
}

pub struct IterMut<'a, T> {
    buf: *mut CircularBuffer<T>,
    idx: usize,
    _marker: PhantomData<&'a mut T>,
}

pub struct IntoIter<T> {
    buf: CircularBuffer<T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let out = self.buf.get(self.idx);
        self.idx += 1;
        out
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: vi garanterar att varje element endast returneras en gång
        let item = unsafe { (*self.buf).get_mut(self.idx) };

        self.idx += 1;
        item
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.buf.pop_front()
    }
}

impl<T> CircularBuffer<T> {
    //core
    /// Wraps around the index to the underlying `data` vec
    fn wrap_index(&self, idx: isize) -> usize {
        let cap = self.data.len() as isize;
        ((idx % cap + cap) % cap) as usize
    }

    /// Calculates logical -> physical index
    /// Checks capacity bounds, not initiated bounds
    ///
    /// params: logical index  (0..self.len)
    ///
    /// returns:
    ///     - Some(usize) physical address in the underlying `data` vec
    ///     - None if the logical index is out of bounds
    fn logical_to_physical(&self, logical: usize) -> Option<usize> {
        if logical >= self.capacity() {
            return None;
        }
        Some(self.wrap_index(self.start as isize + logical as isize))
    }

    pub fn with_capacity(cap: usize) -> Self {
        let mut data = Vec::with_capacity(cap);
        data.resize_with(cap, MaybeUninit::uninit);

        CircularBuffer {
            data,
            start: 0,
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    // Deque ops

    /// Push back,
    ///
    /// returns:
    ///  - Err(T) if the buffer is full
    ///  - Ok(())
    pub fn push_back(&mut self, val: T) -> Result<(), T> {
        //  last element, self.len() - 1, so insert at self.len()
        let maybe_p_index = self.logical_to_physical(self.len());
        if let Some(p_index) = maybe_p_index {
            self.data[p_index].write(val);
            self.len += 1;
            Ok(())
        } else {
            Err(val)
        }
    }

    /// Pop first element returning ownership of T
    ///
    /// returns:
    ///  - Err(T) if the buffer is full
    ///  - Ok(())
    ///
    ///  Api contract,
    ///  - each &mut T will cause undefined behaviour if used after pop of T
    pub fn push_front(&mut self, val: T) -> Result<(), T> {
        // wrap_index does not check if in bounds since modulo, manual check now before
        if self.is_full() {
            return Err(val);
        }

        self.start = self.wrap_index(self.start as isize - 1);
        self.data[self.start].write(val);
        self.len += 1;

        Ok(())
    }

    /// Pop last element returning ownership of T
    ///
    /// returns:
    ///  - None if the buffer is empty
    ///  - Some(T) if the buffer is not empty
    ///
    ///  Api contract,
    ///  - each &mut T will cause undefined behaviour if used after pop of T
    pub fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        //  last element, self.len() - 1
        //SAFETY: since the insertion that made self.len was allowed, we can unwrap
        let p_index = self.logical_to_physical(self.len() - 1).unwrap();

        //SAFETY: we ensure that MaybeUninit is Some because of bookkeping
        let val = unsafe { self.data[p_index].assume_init_read() };
        // we dont move start
        self.len -= 1;
        Some(val)
    }

    /// Pop front
    ///
    /// returns:
    ///  - None if the buffer is empty
    ///  - Some(T) if the buffer is not empty
    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        // uninitialize self.data[self.start]
        let idx = self.logical_to_physical(0).unwrap(); //0 allways inside bounds
                                                        //SAFETY: we ensure that MaybeUninit is Some because of bookkeping
        let val = unsafe { self.data[idx].assume_init_read() }; //move out T

        self.start = self.wrap_index(self.start as isize + 1);
        self.len -= 1;
        Some(val)
    }

    // indexing:

    /// Returns a reference to the element at index i
    ///
    /// returns:
    ///  - None if the index is out of bounds
    ///  - Some(&T) if the index is in bounds
    pub fn get(&self, i: usize) -> Option<&T> {
        //init bounds check
        if i >= self.len {
            return None;
        }
        let p_index = self.logical_to_physical(i).unwrap();
        //SAFETY: we ensure that MaybeUninit is Some because of bookkeping
        let val_ref = unsafe { self.data[p_index].assume_init_ref() };
        Some(val_ref)
    }

    /// Returns a mutable reference to the element at index i
    ///
    /// returns:
    ///  - None if the index is out of bounds
    ///  - Some(&mut T) if the index is in bounds
    ///
    ///  warning:
    ///  - aliasing can occur from i.e the same index is fetched multiple times
    pub unsafe fn get_mut(&mut self, i: usize) -> Option<&mut T> {
        // init bounds check
        if i >= self.len {
            return None;
        }
        let p_index = self.logical_to_physical(i).unwrap();
        //SAFETY: we ensure that MaybeUninit is Some because of bookkeping
        let val_ref = unsafe { self.data[p_index].assume_init_mut() };
        Some(val_ref)
    }

    //iteration:

    pub fn iter(&self) -> Iter<'_, T> {
        Iter { buf: self, idx: 0 }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            buf: self as *mut _,
            idx: 0,
            _marker: PhantomData,
        }
    }

    pub fn make_contiguous(&mut self) {
        // check if contiguous
        if self.len == 0 || self.start == 0 {
            return;
        }

        let mut new_data: Vec<MaybeUninit<T>> = Vec::with_capacity(self.capacity());
        new_data.resize_with(self.capacity(), MaybeUninit::uninit);

        for i in 0..self.len {
            //SAFETY: i < self.len
            let old_idx = self.logical_to_physical(i).unwrap();
            //SAFETY: moving initiated elements
            let val = unsafe { self.data[old_idx].assume_init_read() };
            new_data[i].write(val);
        }

        self.data = new_data;
        self.start = 0;
    }

    pub fn resize(&mut self, new_cap: usize) -> Result<(), ()> {
        if new_cap <= self.capacity() {
            return Err(());
        }

        let mut new_data: Vec<MaybeUninit<T>> = Vec::with_capacity(new_cap);
        new_data.resize_with(new_cap, MaybeUninit::uninit);

        for i in 0..self.len {
            let old_idx = self.logical_to_physical(i).unwrap();
            let val = unsafe { self.data[old_idx].assume_init_read() };
            new_data[i].write(val);
        }

        self.data = new_data;
        self.start = 0;

        Ok(())
    }
}

impl<T> IntoIterator for CircularBuffer<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter { buf: self }
    }
}

impl<T> FromIterator<T> for CircularBuffer<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut buf = CircularBuffer::with_capacity(4);
        for val in iter {
            if buf.is_full() {
                buf.resize(buf.capacity() * 2).expect("capacity growing");
            }
            //SAFETY: fit guaranted
            unsafe {
                buf.push_back(val).unwrap_unchecked();
            }
        }
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_buffer() {
        let buffer: CircularBuffer<u32> = CircularBuffer::with_capacity(10);
        assert!(!buffer.is_full());
        assert!(buffer.len == 0);
        assert!(buffer.start == 0);
    }

    #[test]
    fn test_push_and_pop_back() {
        let mut buffer: CircularBuffer<u32> = CircularBuffer::with_capacity(5);
        assert!(buffer.is_empty());
        assert!(buffer.push_back(1).is_ok());
        assert!(buffer.push_back(2).is_ok());
        assert!(buffer.push_back(3).is_ok());
        assert!(buffer.push_back(4).is_ok());
        assert!(!buffer.is_full());
        assert!(buffer.push_back(5).is_ok());
        assert!(buffer.is_full());
        assert!(buffer.push_back(6).is_err());
        assert!(buffer.push_back(7).is_err());
        assert!(buffer.len() == 5);
        assert!(buffer.is_full());

        for i in 1..6 {
            assert_eq!(buffer.pop_back().unwrap(), 6 - i);
            assert!(!buffer.is_full());
            assert!(buffer.len() == (5 - i) as usize)
        }
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_push_and_pop_front() {
        let mut buffer: CircularBuffer<u32> = CircularBuffer::with_capacity(5);
        assert!(buffer.is_empty());
        assert!(buffer.push_front(1).is_ok());
        assert!(buffer.push_front(2).is_ok());
        assert!(buffer.push_front(3).is_ok());
        assert!(buffer.push_front(4).is_ok());
        assert!(!buffer.is_full());
        assert!(buffer.push_front(5).is_ok());
        assert!(buffer.is_full());
        assert!(buffer.push_front(6).is_err());
        assert!(buffer.push_front(7).is_err());
        assert!(buffer.len() == 5);
        assert!(buffer.is_full());

        for i in 1..6 {
            assert_eq!(buffer.pop_front().unwrap(), 6 - i);
            assert!(!buffer.is_full());
            assert!(buffer.len() == (5 - i) as usize)
        }
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_wraparound_push_pop() {
        let mut buffer: CircularBuffer<u32> = CircularBuffer::with_capacity(5);

        // Fill buffer
        for i in 1..=5 {
            assert!(buffer.push_back(i).is_ok());
        }

        // Pop two from front
        assert_eq!(buffer.pop_front(), Some(1));
        assert_eq!(buffer.pop_front(), Some(2));

        // Push two more, should wrap around
        assert!(buffer.push_back(6).is_ok());
        assert!(buffer.push_back(7).is_ok());

        // Pop all and check order
        let expected = [3, 4, 5, 6, 7];
        for &val in &expected {
            assert_eq!(buffer.pop_front(), Some(val));
        }

        assert!(buffer.is_empty());
    }

    #[test]
    fn test_get() {
        let mut buffer: CircularBuffer<u32> = CircularBuffer::with_capacity(5);

        buffer.push_back(10).unwrap();
        buffer.push_back(20).unwrap();
        buffer.push_back(30).unwrap();

        assert_eq!(buffer.get(0), Some(&10));
        assert_eq!(buffer.get(1), Some(&20));
        assert_eq!(buffer.get(2), Some(&30));
        assert_eq!(buffer.get(3), None); // out of bounds
    }

    #[test]
    fn test_get_mut() {
        let mut buffer: CircularBuffer<u32> = CircularBuffer::with_capacity(5);

        buffer.push_back(1).unwrap();
        buffer.push_back(2).unwrap();
        buffer.push_back(3).unwrap();

        if let Some(x) = unsafe { buffer.get_mut(1) } {
            *x = 42; // mutate in place
        }

        assert_eq!(buffer.get(0), Some(&1));
        assert_eq!(buffer.get(1), Some(&42));
        assert_eq!(buffer.get(2), Some(&3));
        assert_eq!(buffer.get(3), None); // out of bounds
    }

    #[test]
    fn test_iter() {
        let mut buffer = CircularBuffer::with_capacity(5);
        for i in 1..=3 {
            buffer.push_back(i).unwrap();
        }

        let collected: Vec<_> = buffer.iter().copied().collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }

    #[test]
    fn test_iter_mut() {
        let mut buffer = CircularBuffer::with_capacity(5);
        for i in 1..=3 {
            buffer.push_back(i).unwrap();
        }

        for x in buffer.iter_mut() {
            *x *= 2;
        }

        let collected: Vec<_> = buffer.iter().copied().collect();
        assert_eq!(collected, vec![2, 4, 6]);
    }

    #[test]
    fn test_into_iter() {
        let mut buffer = CircularBuffer::with_capacity(5);
        for i in 1..=3 {
            buffer.push_back(i).unwrap();
        }

        let collected: Vec<_> = buffer.into_iter().collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }
    #[test]
    fn test_from_iterator() {
        use std::iter::FromIterator;

        let vec = vec![1, 2, 3, 4, 5];

        let buffer: CircularBuffer<_> = CircularBuffer::from_iter(vec.clone());

        // Kontrollera längd
        assert_eq!(buffer.len(), vec.len());

        // Kontrollera innehåll
        for (i, &val) in vec.iter().enumerate() {
            assert_eq!(buffer.get(i), Some(&val));
        }
    }

    #[test]
    fn test_make_contiguous_no_wrap() {
        let mut buf: CircularBuffer<u32> = CircularBuffer::with_capacity(5);
        for i in 1..=3 {
            buf.push_back(i).unwrap();
        }

        buf.make_contiguous();
        assert_eq!(buf.start, 0);
        for i in 0..buf.len() {
            assert_eq!(buf.get(i), Some(&(i as u32 + 1)));
        }
    }

    #[test]
    fn test_make_contiguous_with_wrap() {
        let mut buf: CircularBuffer<u32> = CircularBuffer::with_capacity(5);
        for i in 1..=5 {
            buf.push_back(i).unwrap();
        }

        buf.pop_front(); // start flyttas
        buf.pop_front(); // wraparound start
        buf.push_back(6).unwrap();
        buf.push_back(7).unwrap();

        buf.make_contiguous();
        assert_eq!(buf.start, 0);
        let expected = [3, 4, 5, 6, 7];
        for (i, &val) in expected.iter().enumerate() {
            assert_eq!(buf.get(i), Some(&val));
        }
    }

    #[test]
    fn test_resize_larger() {
        let mut buf: CircularBuffer<u32> = CircularBuffer::with_capacity(4);
        for i in 1..=4 {
            buf.push_back(i).unwrap();
        }

        let old_capacity = buf.capacity();
        buf.resize(8).unwrap();
        assert!(buf.capacity() >= 8);
        assert_eq!(buf.len(), 4);

        for i in 0..buf.len() {
            assert_eq!(buf.get(i), Some(&(i as u32 + 1)));
        }

        // Push extra elements to test new capacity
        buf.push_back(5).unwrap();
        buf.push_back(6).unwrap();
        assert_eq!(buf.len(), 6);
    }

    #[test]
    fn test_resize_with_wrap() {
        let mut buf: CircularBuffer<u32> = CircularBuffer::with_capacity(4);
        for i in 1..=4 {
            buf.push_back(i).unwrap();
        }

        buf.pop_front();
        buf.pop_front();
        buf.push_back(5).unwrap();
        buf.push_back(6).unwrap(); // wraparound

        buf.resize(8).unwrap();
        assert_eq!(buf.start, 0);
        let expected = [3, 4, 5, 6];
        for (i, &val) in expected.iter().enumerate() {
            assert_eq!(buf.get(i), Some(&val));
        }

        // Test push after resize
        buf.push_back(7).unwrap();
        buf.push_back(8).unwrap();
        assert_eq!(buf.len(), 6);
    }
}
