use std::{cell::RefCell, fmt, rc::Rc};

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

struct Node<T> {
    value: T,
    left: Link<T>,
    right: Link<T>,
}

impl<T: fmt::Debug + Copy> fmt::Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("value", &self.value)
            .field("left", &self.left.as_ref().map(|n| n.borrow().value))
            .field("right", &self.right.as_ref().map(|n| n.borrow().value))
            .finish()
    }
}

#[derive(Default)]
struct BTree<T> {
    root: Link<T>,
}

impl<T> BTree<T> {
    pub fn new() -> Self {
        BTree { root: None }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }
}

impl<T: Ord + Default> BTree<T> {
    //returns Option <PrevNode, val_(is/should be)_to_right of prev, exact_val_found >
    //None if empty
    //if PrevNode none, then we are removing/inserting root
    fn get_prev(&self, val: &T) -> Option<(Link<T>, bool, bool)> {
        if self.is_empty() {
            return None;
        }

        let mut prev: Link<T> = None;
        let mut curr: Link<T> = self.root.clone();
        while let Some(node) = curr.clone() {
            let curr_val = &node.borrow().value;
            if *val > *curr_val {
                // traverse right
                prev = curr.clone();
                if let Some(right) = node.borrow().right.clone() {
                    curr = Some(right.clone());
                } else {
                    curr = None;
                }
            } else if *val < *curr_val {
                // traverse left
                prev = curr.clone();
                if let Some(left) = node.borrow().left.clone() {
                    curr = Some(left.clone());
                } else {
                    curr = None;
                }
            } else {
                // we found the value, prev is the one before, usefull in deletion
                // root?
                let mut is_right = false;
                if let Some(p) = &prev {
                    is_right = *val > p.borrow().value;
                } else {
                    is_right = *val > *curr_val;
                }
                return Some((prev, is_right, true));
            }
        }

        let is_right = *val > prev.as_ref().unwrap().borrow().value;
        Some((prev, is_right, false))
    }

    fn in_ord_successor(node: &Link<T>) -> Link<T> {
        let mut curr = match node {
            Some(rc) => rc.clone(),
            None => return None,
        };

        loop {
            let left_opt = curr.borrow().left.clone();
            if let Some(left) = left_opt {
                curr = left
            } else {
                break;
            }
        }

        Some(curr)
    }

    /// returns
    /// - `true` if success,
    /// - `false` if duplicate val
    pub fn insert(&mut self, val: T) -> bool {
        let prev_meta = self.get_prev(&val);

        let new_node = Some(Rc::new(RefCell::new(Node {
            value: val,
            left: None,
            right: None,
        })));

        match prev_meta {
            Some((_, _, true)) => false, // duplicate
            Some((prev, is_right, false)) => {
                let node = prev.unwrap();
                if is_right {
                    node.borrow_mut().right = new_node;
                } else {
                    node.borrow_mut().left = new_node;
                }
                true
            }
            None => {
                self.root = new_node;
                true
            }
        }
    }

    /// returns
    /// - `true` if success,
    /// - `false` if not found
    pub fn remove(&mut self, val: &T) -> Option<T> {
        let prev_meta = self.get_prev(val);

        match prev_meta {
            Some((_, _, false)) => None, // did not exist
            Some((prev, is_right, true)) => {
                let child = match prev.as_ref() {
                    Some(parent_rc) => {
                        let mut parent_ref = parent_rc.borrow_mut();
                        let target = if is_right {
                            parent_ref.right.take()
                        } else {
                            parent_ref.left.take()
                        };
                        target.expect("child should exist")
                    }
                    None => self.root.take().expect("root should exist"),
                };

                //child exists since duplicate == true
                let mut left_grandchild = child.borrow_mut().left.take();
                let mut right_grandchild = child.borrow_mut().right.take();

                if right_grandchild.is_some() {
                    //promote, either under prev, or as root
                    let right_clone = right_grandchild.as_ref().unwrap().clone(); // Option<&Rc<RefCell<Node<T>>>>
                    let promoted = right_grandchild.take(); // move ownership to prev or root

                    match prev {
                        Some(prev_unwrapped) => {
                            if is_right {
                                prev_unwrapped.borrow_mut().right = promoted;
                            } else {
                                prev_unwrapped.borrow_mut().left = promoted;
                            }
                        }
                        None => {
                            self.root = promoted;
                        }
                    }

                    if left_grandchild.is_some() {
                        let succ = Self::in_ord_successor(&Some(right_clone));
                        succ.unwrap().borrow_mut().left = left_grandchild.take();
                    }
                } else if left_grandchild.is_some() {
                    // right_grandchild None, so 1 child
                    match prev {
                        Some(prev_unwrapped) => {
                            if is_right {
                                prev_unwrapped.borrow_mut().right = left_grandchild.take()
                            } else {
                                prev_unwrapped.borrow_mut().left = left_grandchild.take()
                            }
                        }
                        None => self.root = left_grandchild.take(),
                    }
                }

                // child should now be loose, parent has released it , and we have taken booth
                // right and left
                let mut node = child.borrow_mut();
                Some(std::mem::take(&mut node.value))
            }
            None => None,
        }
    }

    pub fn contains(&self, val: &T) -> bool {
        self.get_prev(val).is_some_and(|(_, _, found)| found)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_btree() {
        let btree: BTree<i32> = BTree::new();
        assert!(btree.root.is_none());
        assert!(btree.is_empty());
    }

    #[test]
    fn test_insert() {
        let mut btree: BTree<i32> = BTree::new();
        let _ = btree.insert(1);
        assert!(btree.contains(&1));
        assert!(!btree.is_empty());

        let nums = vec![
            10, 2, 5, 12, 14, 16, 3, 90, 124, 156, 180, 123, 124, 345, 456,
        ];
        for i in nums {
            let _ = btree.insert(i);
            assert!(btree.contains(&i));
        }
    }

    #[test]
    fn test_insert_big_dataset() {
        let mut btree: BTree<i32> = BTree::new();
        let big_dataset: Vec<i32> = (0..10000).collect();
        let mut iter = big_dataset.split(|n| *n == 5000);

        // två slice
        let first = iter.next().unwrap(); // &[0..5000]
        let second = iter.next().unwrap(); // &[5001..10000]

        // scrambled
        let scrambled: Vec<i32> = first
            .iter()
            .rev()
            .copied()
            .chain(second.iter().copied())
            .collect();

        for i in scrambled {
            let _ = btree.insert(i);
            assert!(btree.contains(&i));
        }
    }

    #[test]
    fn test_insert_bad() {
        let mut btree: BTree<i32> = BTree::new();
        let _ = btree.insert(1);
        assert!(!btree.insert(1));
    }

    #[test]
    fn test_remove_leaf() {
        let mut tree = BTree::new();
        tree.insert(10);
        tree.insert(5);
        tree.insert(15);

        assert!(tree.remove(&5).is_some());
        assert!(!tree.contains(&5));
    }

    #[test]
    fn test_remove_one_child() {
        let mut tree = BTree::new();
        tree.insert(10);
        tree.insert(5);
        tree.insert(3); // left child of 5

        assert!(tree.remove(&5).is_some());
        assert!(!tree.contains(&5));
        assert!(tree.contains(&3));
    }

    #[test]
    fn test_remove_two_children() {
        let mut tree = BTree::new();
        tree.insert(10);
        tree.insert(5);
        tree.insert(15);
        tree.insert(12);
        tree.insert(18);

        assert!(tree.remove(&15).is_some());
        assert!(!tree.contains(&15));
        assert!(tree.contains(&12));
        assert!(tree.contains(&18));
    }

    #[test]
    fn test_remove_root() {
        let mut tree = BTree::new();
        tree.insert(10);
        tree.insert(5);
        tree.insert(15);

        assert!(tree.remove(&10).is_some());
        assert!(!tree.contains(&10));
        assert!(tree.contains(&5));
        assert!(tree.contains(&15));
    }

    #[test]
    fn test_remove_from_scrambled_dataset() {
        let nums = vec![
            10, 2, 5, 12, 14, 16, 3, 90, 124, 156, 180, 123, 124, 345, 456,
        ];
        let mut tree = BTree::new();
        for &n in &nums {
            tree.insert(n);
        }

        // Use a HashSet to deduplicate
        let mut unique_nums: Vec<i32> = nums
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        unique_nums.sort();

        for &n in &unique_nums {
            assert!(tree.remove(&n).is_some(), "Failed to remove {}", n);
            assert!(!tree.contains(&n));
        }

        assert!(tree.is_empty());
    }
}
