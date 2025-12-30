struct CircularBuffer<T> {
    data: Vec<T>,
    start: usize,
    len: usize,
}

pub struct Iter<'a, T> {
    buf: &'a CircularBuffer<T>,
    idx: usize,
    remaining: usize,
}

pub struct IterMut<'a, T> {
    buf: &'a mut CircularBuffer<T>,
    idx: usize,
    remaining: usize,
}

pub struct IntoIter<T> {
    data: Vec<T>,
    idx: usize,
    remaining: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<T> CircularBuffer<T> {
    //core

    /// Calculates logical -> physical index
    fn wrap_index(&self, idx: isize) -> usize {
        let cap = self.data.len() as isize;
        ((idx % cap + cap) % cap) as usize
    }

    fn with_capacity(cap: usize) -> Self {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }

    fn capacity(&self) -> usize {
        todo!()
    }

    fn is_empty(&self) -> bool {
        todo!()
    }

    fn is_full(&self) -> bool {
        todo!()
    }

    // Deque ops
    fn push_back(val: T) -> Result<(), T> {
        todo!()
    }

    fn push_front(val: T) -> Result<(), T> {
        todo!()
    }

    fn pop_back() -> Option<T> {
        todo!()
    }

    fn pop_front() -> Option<T> {
        todo!()
    }

    // indexing:

    fn get(&self, i: usize) -> Option<&T> {
        todo!()
    }

    fn get_mut(&mut self, i: usize) -> Option<&mut T> {
        todo!()
    }

    //iteration:

    fn iter(&self) -> Iter<'_, T> {
        todo!()
    }

    fn iter_mut(&mut self) -> IterMut<'_, T> {
        todo!()
    }

    fn as_slices(&self) -> (&[T], &[T]) {
        todo!()
    }

    fn make_contiguous() {
        todo!();
    }

    fn resize(new_cap: usize) {
        todo!();
    }
}

impl<T> IntoIterator for CircularBuffer<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_buffer() {
        let buffer: CircularBuffer<u32> = CircularBuffer::with_capacity(10);
    }
}
