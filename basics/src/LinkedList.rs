#![allow(dead_code)]

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    value: T,
    next: Link<T>,
}

pub struct LinkedList<T> {
    head: Link<T>,
    tail: *mut Node<T>,
    pub length: usize,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        LinkedList {
            head: None,
            tail: std::ptr::null_mut(),
            length: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0 || self.head.is_none()
    }

    pub fn push_front(&mut self, data: T) {
        let mut new_node = Box::new(Node {
            value: data,
            next: self.head.take(),
        });

        let raw_node: *mut _ = &mut *new_node;
        if self.tail.is_null() {
            self.tail = raw_node;
        }

        self.head = Some(new_node);
        self.length += 1;
    }

    pub fn push_tail(&mut self, value: T) {
        let mut new_node = Box::new(Node { value, next: None });
        let raw_node: *mut _ = &mut *new_node;

        if self.tail.is_null() {
            self.head = Some(new_node);
        } else {
            unsafe {
                (*self.tail).next = Some(new_node);
            }
        }

        self.tail = raw_node;
        self.length += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            let Node { value, next } = *node;
            self.head = next;

            if self.head.is_none() {
                self.tail = std::ptr::null_mut();
            }

            self.length -= 1;
            value
        })
    }

    pub fn pop_tail(&mut self) -> Option<T> {
        if self.head.is_none() {
            return None;
        }

        // Single element case
        if self.head.as_ref().unwrap().next.is_none() {
            return self.pop_front();
        }

        // Find the second-to-last node
        let mut current = self.head.as_mut().unwrap();
        while current.next.as_ref().map_or(false, |n| n.next.is_some()) {
            current = current.next.as_mut().unwrap();
        }

        // current is now the second-to-last node
        let last_node = current.next.take().unwrap();
        self.tail = &mut **current as *mut Node<T>;
        self.length -= 1;
        Some(last_node.value)
    }

    pub fn peek_front(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.value)
    }

    pub fn peek_tail(&self) -> Option<&T> {
        if self.tail.is_null() {
            None
        } else {
            unsafe { Some(&(*self.tail).value) }
        }
    }

    pub fn peek_front_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| &mut node.value)
    }

    pub fn peek_tail_mut(&mut self) -> Option<&mut T> {
        if self.tail.is_null() {
            None
        } else {
            unsafe { Some(&mut (*self.tail).value) }
        }
    }

    pub fn clear(&mut self) {
        while self.pop_front().is_some() {}
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            next: self.head.as_deref_mut(),
        }
    }
}

pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.value
        })
    }
}

impl<T> IntoIterator for LinkedList<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}

pub struct IntoIter<T>(LinkedList<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }
}

pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| {
            self.next = node.next.as_deref_mut();
            &mut node.value
        })
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}