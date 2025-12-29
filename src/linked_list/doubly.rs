use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

#[derive(Default)]
struct Node<T> {
    value: T,
    next: Link<T>,
    prev: Link<T>,
}

#[derive(Default)]
struct DoublyLinkedList<T> {
    head: Link<T>,
    tail: Link<T>,
    len: usize,
}

impl<T: Copy> DoublyLinkedList<T> {
    pub fn new() -> Self {
        DoublyLinkedList {
            head: None,
            tail: None,
            len: 0,
        }
    }

    pub fn push_front(&mut self, val: T) {
        let is_empty = self.is_empty();
        let tmp = self.head.take();
        let link: Link<T> = Some(Rc::new(RefCell::new(Node {
            value: val,
            next: tmp.clone(),
            prev: None,
        })));

        // old head needs to point backwards
        if !is_empty {
            tmp.unwrap().borrow_mut().prev = link.clone();
        }

        self.head = link.clone();

        // corner case
        if self.is_empty() {
            self.tail = link.clone();
        }

        self.len += 1;
    }

    pub fn push_back(&mut self, val: T) {
        let is_empty = self.is_empty();
        let tmp = self.tail.take();

        let link: Link<T> = Some(Rc::new(RefCell::new(Node {
            value: val,
            next: None,
            prev: tmp.clone(),
        })));

        // if list was not empty, the prev element need to point to new tail
        if !is_empty {
            tmp.unwrap().borrow_mut().next = link.clone();
        }

        self.tail = link.clone();

        //corner case
        if self.is_empty() {
            self.head = link.clone();
        }

        self.len += 1;
    }

    /// Returns a ref guard to the front element
    pub fn peek_front<'a>(&'a self) -> Option<Ref<'a, T>> {
        self.head
            .as_ref()
            .map(|node| Ref::map(node.borrow(), |n| &n.value))
    }

    pub fn peek_back<'a>(&'a self) -> Option<Ref<'a, T>> {
        self.tail
            .as_ref()
            .map(|node| Ref::map(node.borrow(), |n| &n.value))
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|head_rc| {
            let next = head_rc.borrow().next.clone();
            self.head = next;

            //we pointed backwards to the head that is now removed, we need to set it to None.
            if let Some(new_head) = &self.head {
                new_head.borrow_mut().prev = None;
            } else {
                self.tail = None;
            }

            self.len -= 1;

            head_rc.borrow().value
        })
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.take().map(|tail_rc| {
            //cut out tail_rc from the linked list
            let prev = tail_rc.borrow().prev.clone();
            self.tail = prev;

            if let Some(new_tail) = &self.tail {
                new_tail.borrow_mut().next = None;
            } else {
                self.head = None;
            }

            self.len -= 1;

            tail_rc.borrow().value
        })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn iter(&self) -> DoublyIter<'_, T> {
        DoublyIter {
            next: self.head.clone(),
            marker: std::marker::PhantomData,
        }
    }
}

impl<T: Copy> IntoIterator for DoublyLinkedList<T> {
    type Item = T;
    type IntoIter = DoublyIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        DoublyIntoIter {
            next: self.head,
            prev: self.tail,
        }
    }
}

impl<T: Copy> FromIterator<T> for DoublyLinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = DoublyLinkedList::new();
        for item in iter {
            list.push_back(item);
        }
        list
    }
}

pub struct DoublyIntoIter<T> {
    next: Option<Rc<RefCell<Node<T>>>>,
    prev: Option<Rc<RefCell<Node<T>>>>,
}

impl<T: Copy> Iterator for DoublyIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|rc_node| {
            let node = rc_node.borrow();
            self.next = node.next.clone();
            node.value
        })
    }
}

impl<T: Copy> DoubleEndedIterator for DoublyIntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.prev.take().map(|rc_node| {
            let node = rc_node.borrow();
            self.prev = node.prev.clone();
            node.value
        })
    }
}

pub struct DoublyIter<'a, T> {
    next: Option<Rc<RefCell<Node<T>>>>,
    marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T: Copy> Iterator for DoublyIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|rc_node| {
            let node = rc_node.borrow();
            self.next = node.next.clone();
            node.value
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let dll = DoublyLinkedList::<u32>::new();
        assert!(dll.head.is_none());
        assert!(dll.tail.is_none());
        assert_eq!(dll.len, 0);
    }

    #[test]
    fn test_push_front() {
        let mut dll = DoublyLinkedList::<u32>::new();

        dll.push_front(1);
        assert!(!dll.is_empty());
        assert_eq!(dll.len(), 1);
        assert_eq!(*dll.peek_front().unwrap(), 1);
        assert_eq!(*dll.peek_back().unwrap(), 1);

        for i in 2..10 {
            dll.push_front(i);
            assert_eq!(dll.len(), i as usize);
            assert_eq!(*dll.peek_front().unwrap(), i);
            assert_eq!(*dll.peek_back().unwrap(), 1);
        }
    }

    #[test]
    fn test_push_back() {
        let mut dll = DoublyLinkedList::<u32>::new();

        dll.push_back(1);
        assert!(!dll.is_empty());
        assert_eq!(dll.len(), 1);
        assert_eq!(*dll.peek_front().unwrap(), 1);
        assert_eq!(*dll.peek_back().unwrap(), 1);

        for i in 2..10 {
            dll.push_back(i);
            assert_eq!(dll.len(), i as usize);
            assert_eq!(*dll.peek_back().unwrap(), i);
            assert_eq!(*dll.peek_front().unwrap(), 1);
        }
    }

    #[test]
    fn test_pop_front() {
        let mut dll = DoublyLinkedList::<u32>::new();

        for i in 0..10 {
            assert_eq!(dll.len(), i as usize);
            dll.push_back(i);
            assert_eq!(*dll.peek_back().unwrap(), i);
            assert_eq!(*dll.peek_front().unwrap(), 0);
        }

        for i in 0..10 {
            assert_eq!(dll.pop_front(), Some(i));
        }
        assert!(dll.is_empty());
        assert_eq!(dll.len(), 0);
    }

    #[test]
    fn test_pop_back() {
        let mut dll = DoublyLinkedList::<u32>::new();

        for i in 0..10 {
            assert_eq!(dll.len(), i as usize);
            dll.push_front(i);
            assert_eq!(*dll.peek_back().unwrap(), 0);
            assert_eq!(*dll.peek_front().unwrap(), i);
        }

        for i in 0..10 {
            assert_eq!(dll.pop_back(), Some(i));
        }
        assert!(dll.is_empty());
        assert_eq!(dll.len(), 0);
    }

    #[test]
    fn test_iter() {
        let mut dll: DoublyLinkedList<i32> = (0..5).collect();
        let mut iter = dll.iter();

        let mut collected = Vec::new();
        while let Some(val) = iter.next() {
            collected.push(val);
        }

        assert_eq!(collected, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_into_iter() {
        let dll: DoublyLinkedList<i32> = (0..5).collect();
        let iter = dll.into_iter();
        let collected: Vec<_> = iter.collect();
        assert_eq!(collected, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_double_ended_iter_forward() {
        let dll: DoublyLinkedList<i32> = (0..5).collect();
        let mut iter = dll.into_iter();

        let mut collected = Vec::new();
        collected.push(iter.next().unwrap()); // 0
        collected.push(iter.next().unwrap()); // 1
        assert_eq!(collected, vec![0, 1]);
    }

    #[test]
    fn test_double_ended_iter_backward() {
        let dll: DoublyLinkedList<i32> = (0..5).collect();
        let mut iter = dll.into_iter();

        let last = iter.next_back().unwrap();
        let second_last = iter.next_back().unwrap();
        assert_eq!(vec![second_last, last], vec![3, 4]);
    }

    #[test]
    fn test_double_ended_iter_mixed() {
        let dll: DoublyLinkedList<i32> = (0..5).collect();
        let mut iter = dll.into_iter();

        let f1 = iter.next().unwrap();
        let b1 = iter.next_back().unwrap();
        let f2 = iter.next().unwrap();
        let b2 = iter.next_back().unwrap();
        let f3 = iter.next().unwrap(); // should be the middle element

        assert_eq!(vec![f1, f2, f3, b2, b1], vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_from_iterator() {
        let dll: DoublyLinkedList<_> = (0..5).collect();
        let collected: Vec<_> = dll.iter().collect();
        assert_eq!(collected, vec![0, 1, 2, 3, 4]);
    }
}
