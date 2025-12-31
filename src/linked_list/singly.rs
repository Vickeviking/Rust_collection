//! A Singly linked list, implemented in safe rust with smart pointers
//! A LIFO structure with push, pop and peek-front, essentially a stack.
type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    value: T,
    next: Link<T>,
}

#[derive(Default)]
struct SinglyLinkedList<T> {
    root: Link<T>,
}

impl<T> SinglyLinkedList<T> {
    pub fn new() -> Self {
        SinglyLinkedList { root: None }
    }

    pub fn push_front(&mut self, val: T) {
        //we need to move out root
        let tmp = self.root.take();
        // append tmp to new root
        self.root = Some(Box::new(Node {
            value: val,
            next: tmp,
        }));
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.root.take().map(|boxed| {
            //root node
            let node = *boxed;
            self.root = node.next;
            node.value
        })
    }

    pub fn peek_front(&self) -> Option<&T> {
        self.root.as_ref().map(|node| &node.value)
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    // by reference, non owning
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            curr: self.root.as_deref(),
        }
    }
}

//=== Iterators ===
impl<T> IntoIterator for SinglyLinkedList<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { curr: self.root }
    }
}

impl<T> FromIterator<T> for SinglyLinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = SinglyLinkedList::new();
        for item in iter {
            list.push_front(item); // LIFO: senaste element blir toppen
        }
        list
    }
}

struct Iter<'a, T> {
    curr: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.curr?; //take current
        self.curr = node.next.as_deref();
        Some(&node.value)
    }
}

pub struct IntoIter<T> {
    curr: Link<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.curr.take().map(|boxed| {
            let node = *boxed;
            self.curr = node.next;
            node.value
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push() {
        let mut sll = SinglyLinkedList::<u16>::new();

        assert!(sll.pop_front().is_none());
        assert!(sll.peek_front().is_none());
        assert!(sll.is_empty());

        // 1 item
        sll.push_front(1);
        assert_eq!(sll.peek_front(), Some(&1));
        assert!(!sll.is_empty());
        assert_eq!(sll.pop_front(), Some(1));
        assert!(sll.pop_front().is_none());
        assert!(sll.peek_front().is_none());

        // insert 100 items
        for i in 0..100 {
            sll.push_front(i);
            assert_eq!(sll.peek_front(), Some(&i));
        }

        for i in (0..100).rev() {
            assert_eq!(sll.pop_front(), Some(i));
        }
        assert!(sll.is_empty());
    }

    #[test]
    fn test_iter() {
        let mut sll = SinglyLinkedList::new();
        for i in 0..5 {
            sll.push_front(i);
        }

        let mut sll_ref_iter = sll.iter();
        for i in (0..5).rev() {
            assert_eq!(*(sll_ref_iter.next().unwrap()), i)
        }

        let collected: Vec<_> = sll.iter().copied().collect();
        assert_eq!(collected, vec![4, 3, 2, 1, 0]);

        // Konsumerande
        sll = (0..=3).map(|x| x * 2).collect();
        let sll_iter = sll.into_iter();
        assert_eq!(sll_iter.take(2).collect::<Vec<i32>>(), vec![6, 4]);
    }
}
