use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ptr::NonNull;

pub struct Node<K, V> {
    k: K,
    v: V,
    prev: Option<NonNull<Node<K, V>>>,
    next: Option<NonNull<Node<K, V>>>,
}

struct KeyRef<K, V>(NonNull<Node<K, V>>);

impl<K: Hash + Eq, V> Borrow<K> for KeyRef<K, V> {
    fn borrow(&self) -> &K {
        unsafe { &self.0.as_ref().k }
    }
}

impl<K: Hash, V> Hash for KeyRef<K, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe { self.0.as_ref().k.hash(state) }
    }
}

impl <K: Eq, V> PartialEq for KeyRef<K, V> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.0.as_ref().k.eq(&other.0.as_ref().k) }
    }
}

impl<K: Eq, V> Eq for KeyRef<K, V> {}



impl<K, V> Node<K, V> {
    fn new(k: K, v: V) -> Self {
        Node { 
            k: k, 
            v: v,
            prev: None,
            next: None,
        }
    }
}




impl<K: Hash, V> Hash for Node<K, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.k.hash(state);
    }
}

impl<K, V> Borrow<K> for Node<K, V> {
    fn borrow(&self) -> &K {
        &self.k
    }
}

impl<K: Eq, V> PartialEq for Node<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.k.eq(&other.k)
    }
}


impl<K: Eq, V> Eq for Node<K, V> {}


pub struct LruCache<K, V> {
    head: Option<NonNull<Node<K, V>>>,
    tail: Option<NonNull<Node<K, V>>>,
    map: HashMap<KeyRef<K, V>, NonNull<Node<K, V>>>,
    cap: usize,
    marker: PhantomData<Node<K, V>>,
}

impl<K: Hash + Eq + PartialEq, V> LruCache<K, V> {
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0);
        Self {
            head: None,
            tail: None,
            map: HashMap::new(),
            cap,
            marker: PhantomData,
        }
    }

    pub fn put(&mut self, k: K, v: V) -> Option<V> {
        let node = Box::leak(Box::new(Node::new(k, v))).into();

        let old_node = self.map.remove(&KeyRef(node)).map(|node| {
            self.detach(node);
            node
        });

        if self.map.len() >= self.cap {
            let tail = self.tail.unwrap();
            self.detach(tail);
            self.map.remove(&KeyRef(tail));
        }

        self.attach(node);
        self.map.insert(KeyRef(node), node);
        old_node.map(|node| unsafe {
            let node = Box::from_raw(node.as_ptr());
            node.v
        })
    }

    pub fn get(&mut self, k: &K) -> Option<&V> {
        if let Some(node) = self.map.get(k) {
            let node = *node;
            self.detach(node);
            self.attach(node);
            unsafe { Some(&node.as_ref().v)}
        } else {
            None
        }
    }

    pub fn detach(&mut self, mut node: NonNull<Node<K, V>>) {
        unsafe {
            match node.as_mut().prev {
                Some(mut prev) => {
                    prev.as_mut().next = node.as_ref().next;
                }
                None => {
                    self.head = node.as_ref().next;
                }
            }

            match node.as_mut().next {
                Some(mut next) => {
                    next.as_mut().prev = node.as_ref().prev;
                }
                None => {
                    self.tail = node.as_ref().prev;
                }
            }

            node.as_mut().prev = None;
            node.as_mut().next = None;
        }
    }

    fn attach(&mut self, mut node: NonNull<Node<K, V>>) {
        match self.head {
            Some(mut head) => {
                unsafe {
                    head.as_mut().prev = Some(node);
                    node.as_mut().next = Some(head);
                    node.as_mut().prev = None;
                }
                self.head = Some(node);
            }
            None => {
                unsafe {
                    node.as_mut().prev = None;
                    node.as_mut().next = None;
                }
                self.head = Some(node);
                self.tail = Some(node);
            }
        }
    }
}

impl<K, V> Drop for LruCache<K, V> {
    fn drop(&mut self) {
        while let Some(node) = self.head.take() {
            unsafe {
                self.head = node.as_ref().next;
                drop(node.as_ptr());
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn it_works() {
        let node = Node::new("k", "v");
        assert_eq!(node.k, "k");
        assert_eq!(node.v, "v");
    }

    #[test]
    fn it_works2() {
        let mut m = HashMap::new();
        m.insert(Node::new(1, 1), 1);

        let v = m.get(&1);
        assert_eq!(v, Some(&1));
    }


    #[test]
    fn it_works3() {
        let mut lru = LruCache::new(3);
        println!("it's ok");
        assert_eq!(lru.put(1, 10), None);
        println!("put 1");
        assert_eq!(lru.put(2, 20), None);
        println!("put 2");
        assert_eq!(lru.put(3, 30), None);
        println!("put 3");
        assert_eq!(lru.get(&1), Some(&10));
        println!("get 1");
        // assert_eq!(lru.put(2, 200), Some(20));
        // println!("put 2 again then we should get the order value of key 2 : 20");
        // assert_eq!(lru.put(4, 40), None);
        // println!("put 4 , should delete the ordest value from map ");
        // assert_eq!(lru.get(&2), Some(&200));
        // println!("get 2");
    }
}
